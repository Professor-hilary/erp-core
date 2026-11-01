// src/features/payroll/calculation.rs

use bigdecimal::BigDecimal;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

fn zero() -> BigDecimal {
    BigDecimal::from(0)
}

fn round_money(value: &BigDecimal, scale: i64) -> BigDecimal {
    value.round(scale)
}

fn inclusive_days(start: NaiveDate, end: NaiveDate) -> i64 {
    (end - start).num_days() + 1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxBand {
    pub lower_bound: BigDecimal,
    pub upper_bound: Option<BigDecimal>,
    pub base_tax: BigDecimal,
    pub marginal_rate: BigDecimal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComponentKind {
    Earning,
    Deduction,
    EmployerCost,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CalculationMethod {
    Fixed,
    PercentOfBasic,
    PercentOfGross,
    PercentOfTaxableEarnings,
    Manual,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInput {
    pub component_uuid: Option<Uuid>,
    pub code: String,
    pub name: String,

    pub kind: ComponentKind,
    pub method: CalculationMethod,

    pub amount: BigDecimal,
    pub rate: Option<BigDecimal>,

    pub taxable: bool,
    pub included_in_nssf_base: bool,

    pub sequence_no: i16,
    pub rounding_scale: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeTaxProfile {
    pub tax_residency: String,
    pub nssf_covered: bool,
    pub nssf_exempt: bool,
    pub date_of_birth: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollInput {
    pub employee_uuid: Uuid,

    pub basic_salary: BigDecimal,

    pub period_start: NaiveDate,
    pub period_end: NaiveDate,

    pub contract_start: NaiveDate,
    pub contract_end: Option<NaiveDate>,

    pub components: Vec<ComponentInput>,

    pub tax_profile: EmployeeTaxProfile,
    pub tax_bands: Vec<TaxBand>,

    pub nssf_employee_rate: BigDecimal,
    pub nssf_employer_rate: BigDecimal,

    pub nssf_minimum_age: Option<u8>,
    pub nssf_maximum_age: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatedItem {
    pub component_uuid: Option<Uuid>,

    pub code: String,
    pub name: String,

    pub kind: ComponentKind,
    pub method: CalculationMethod,

    pub basis_amount: BigDecimal,
    pub rate: Option<BigDecimal>,

    pub unrounded_amount: BigDecimal,
    pub amount: BigDecimal,

    pub taxable: bool,
    pub included_in_nssf_base: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollResult {
    pub employee_uuid: Uuid,

    pub basic_pay: BigDecimal,
    pub gross_pay: BigDecimal,
    pub taxable_earnings: BigDecimal,

    pub paye: BigDecimal,

    pub employee_nssf: BigDecimal,
    pub employer_nssf: BigDecimal,

    pub other_deductions: BigDecimal,
    pub employer_cost_total: BigDecimal,

    pub net_pay: BigDecimal,

    pub items: Vec<CalculatedItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayrollCalculationError {
    InvalidPeriod,
    InvalidContractPeriod,
    NegativeBasicSalary,
    MissingTaxBands,
    InvalidTaxBands,
    InvalidTaxResidency,
    InvalidComponentAmount(String),
    InvalidComponentRate(String),
    DuplicateComponent(String),
    MissingDateOfBirth,
    InvalidNssfRate,
    NegativeNetPay,
}

impl std::fmt::Display for PayrollCalculationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PayrollCalculationError {}

pub fn prorate_salary(
    salary: &BigDecimal,
    period_start: NaiveDate,
    period_end: NaiveDate,
    contract_start: NaiveDate,
    contract_end: Option<NaiveDate>,
) -> Result<BigDecimal, PayrollCalculationError> {
    if period_end < period_start {
        return Err(PayrollCalculationError::InvalidPeriod);
    }

    if contract_end.is_some_and(|end| end < contract_start) {
        return Err(PayrollCalculationError::InvalidContractPeriod);
    }

    if salary < &zero() {
        return Err(PayrollCalculationError::NegativeBasicSalary);
    }

    let overlap_start = period_start.max(contract_start);
    let overlap_end = contract_end
        .map(|end| period_end.min(end))
        .unwrap_or(period_end);

    if overlap_end < overlap_start {
        return Ok(zero());
    }

    let period_days = inclusive_days(period_start, period_end);
    let eligible_days = inclusive_days(overlap_start, overlap_end);

    Ok(round_money(
        &(salary * BigDecimal::from(eligible_days)
            / BigDecimal::from(period_days)),
        2,
    ))
}

pub fn progressive_tax(
    taxable_income: &BigDecimal,
    bands: &[TaxBand],
) -> Result<BigDecimal, PayrollCalculationError> {
    if taxable_income <= &zero() {
        return Ok(zero());
    }

    if bands.is_empty() {
        return Err(PayrollCalculationError::MissingTaxBands);
    }

    let mut ordered = bands.to_vec();

    ordered.sort_by(|a, b| a.lower_bound.cmp(&b.lower_bound));

    for (index, band) in ordered.iter().enumerate() {
        if band.lower_bound < zero()
            || band.base_tax < zero()
            || band.marginal_rate < zero()
            || band.marginal_rate > BigDecimal::from(1)
        {
            return Err(PayrollCalculationError::InvalidTaxBands);
        }

        if let Some(upper) = &band.upper_bound {
            if upper <= &band.lower_bound {
                return Err(PayrollCalculationError::InvalidTaxBands);
            }

            if let Some(next) = ordered.get(index + 1) {
                if next.lower_bound != *upper {
                    return Err(PayrollCalculationError::InvalidTaxBands);
                }
            }
        }
    }

    let band = ordered
        .iter()
        .rev()
        .find(|band| taxable_income >= &band.lower_bound)
        .ok_or(PayrollCalculationError::InvalidTaxBands)?;

    let taxable_in_band = match &band.upper_bound {
        Some(upper) => taxable_income.clone().min(upper.clone())
            - &band.lower_bound,
        None => taxable_income - &band.lower_bound,
    };

    let tax = &band.base_tax
        + (taxable_in_band * &band.marginal_rate);

    Ok(round_money(&tax, 2))
}

fn calculate_component(
    component: &ComponentInput,
    basic: &BigDecimal,
    gross_before: &BigDecimal,
    taxable_before: &BigDecimal,
) -> Result<(BigDecimal, BigDecimal), PayrollCalculationError> {
    if component.amount < zero() {
        return Err(
            PayrollCalculationError::InvalidComponentAmount(
                component.code.clone(),
            ),
        );
    }

    let (basis, amount) = match component.method {
        CalculationMethod::Fixed
        | CalculationMethod::Manual
        | CalculationMethod::External => (
            component.amount.clone(),
            component.amount.clone(),
        ),

        CalculationMethod::PercentOfBasic => {
            let rate = component.rate.as_ref().ok_or_else(|| {
                PayrollCalculationError::InvalidComponentRate(
                    component.code.clone(),
                )
            })?;

            if rate < &zero() {
                return Err(
                    PayrollCalculationError::InvalidComponentRate(
                        component.code.clone(),
                    ),
                );
            }

            (basic.clone(), basic * rate)
        }

        CalculationMethod::PercentOfGross => {
            let rate = component.rate.as_ref().ok_or_else(|| {
                PayrollCalculationError::InvalidComponentRate(
                    component.code.clone(),
                )
            })?;

            if rate < &zero() {
                return Err(
                    PayrollCalculationError::InvalidComponentRate(
                        component.code.clone(),
                    ),
                );
            }

            (gross_before.clone(), gross_before * rate)
        }

        CalculationMethod::PercentOfTaxableEarnings => {
            let rate = component.rate.as_ref().ok_or_else(|| {
                PayrollCalculationError::InvalidComponentRate(
                    component.code.clone(),
                )
            })?;

            if rate < &zero() {
                return Err(
                    PayrollCalculationError::InvalidComponentRate(
                        component.code.clone(),
                    ),
                );
            }

            (taxable_before.clone(), taxable_before * rate)
        }
    };

    Ok((
        basis,
        round_money(&amount, component.rounding_scale),
    ))
}

fn age_at(date_of_birth: NaiveDate, date: NaiveDate) -> i32 {
    let mut age = date.year() - date_of_birth.year();

    if (date.month(), date.day())
        < (date_of_birth.month(), date_of_birth.day())
    {
        age -= 1;
    }

    age
}

fn nssf_is_applicable(
    profile: &EmployeeTaxProfile,
    period_end: NaiveDate,
    minimum_age: Option<u8>,
    maximum_age: Option<u8>,
) -> Result<bool, PayrollCalculationError> {
    if !profile.nssf_covered || profile.nssf_exempt {
        return Ok(false);
    }

    let dob = profile.date_of_birth
        .ok_or(PayrollCalculationError::MissingDateOfBirth)?;

    let age = age_at(dob, period_end);

    if minimum_age.is_some_and(|min| age < i32::from(min)) {
        return Ok(false);
    }

    if maximum_age.is_some_and(|max| age > i32::from(max)) {
        return Ok(false);
    }

    Ok(true)
}

pub fn calculate_payroll(
    input: &PayrollInput,
) -> Result<PayrollResult, PayrollCalculationError> {
    if input.period_end < input.period_start {
        return Err(PayrollCalculationError::InvalidPeriod);
    }

    if input.tax_profile.tax_residency != "RESIDENT"
        && input.tax_profile.tax_residency != "NON_RESIDENT"
    {
        return Err(PayrollCalculationError::InvalidTaxResidency);
    }

    if input.nssf_employee_rate < zero()
        || input.nssf_employer_rate < zero()
        || input.nssf_employee_rate > BigDecimal::from(1)
        || input.nssf_employer_rate > BigDecimal::from(1)
    {
        return Err(PayrollCalculationError::InvalidNssfRate);
    }

    let basic = prorate_salary(
        &input.basic_salary,
        input.period_start,
        input.period_end,
        input.contract_start,
        input.contract_end,
    )?;

    let mut components = input.components.clone();

    components.sort_by(|a, b| {
        a.sequence_no
            .cmp(&b.sequence_no)
            .then_with(|| a.code.cmp(&b.code))
    });

    let mut seen = HashSet::new();

    let mut gross = basic.clone();
    let mut taxable = basic.clone();
    let mut nssf_base = basic.clone();

    let mut deductions = zero();
    let mut employer_costs = zero();

    let mut items = Vec::new();

    for component in &components {
        let normalized_code = component.code.trim().to_uppercase();

        if normalized_code.is_empty() || !seen.insert(normalized_code) {
            return Err(
                PayrollCalculationError::DuplicateComponent(
                    component.code.clone(),
                ),
            );
        }

        let (basis, amount) = calculate_component(
            component,
            &basic,
            &gross,
            &taxable,
        )?;

        match component.kind {
            ComponentKind::Earning => {
                gross += &amount;

                if component.taxable {
                    taxable += &amount;
                }

                if component.included_in_nssf_base {
                    nssf_base += &amount;
                }
            }

            ComponentKind::Deduction => {
                deductions += &amount;
            }

            ComponentKind::EmployerCost => {
                employer_costs += &amount;
            }
        }

        items.push(CalculatedItem {
            component_uuid: component.component_uuid,
            code: component.code.clone(),
            name: component.name.clone(),
            kind: component.kind,
            method: component.method,
            basis_amount: basis,
            rate: component.rate.clone(),
            unrounded_amount: amount.clone(),
            amount,
            taxable: component.taxable,
            included_in_nssf_base: component.included_in_nssf_base,
        });
    }

    let paye = progressive_tax(&taxable, &input.tax_bands)?;

    let nssf_applicable = nssf_is_applicable(
        &input.tax_profile,
        input.period_end,
        input.nssf_minimum_age,
        input.nssf_maximum_age,
    )?;

    let employee_nssf = if nssf_applicable {
        round_money(
            &(&nssf_base * &input.nssf_employee_rate),
            2,
        )
    } else {
        zero()
    };

    let employer_nssf = if nssf_applicable {
        round_money(
            &(&nssf_base * &input.nssf_employer_rate),
            2,
        )
    } else {
        zero()
    };

    let net_pay = round_money(
        &(&gross - &paye - &employee_nssf - &deductions),
        2,
    );

    if net_pay < zero() {
        return Err(PayrollCalculationError::NegativeNetPay);
    }

    let employer_cost_total = round_money(
        &(&gross + &employer_nssf + &employer_costs),
        2,
    );

    Ok(PayrollResult {
        employee_uuid: input.employee_uuid,
        basic_pay: basic,
        gross_pay: round_money(&gross, 2),
        taxable_earnings: round_money(&taxable, 2),
        paye,
        employee_nssf,
        employer_nssf,
        other_deductions: round_money(&deductions, 2),
        employer_cost_total,
        net_pay,
        items,
    })
}
