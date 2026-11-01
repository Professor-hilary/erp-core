// src/features/payroll/calculation.rs

use bigdecimal::BigDecimal;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

fn money(value: &str) -> BigDecimal {
    BigDecimal::from_str(value).expect("valid decimal constant")
}

fn zero() -> BigDecimal {
    BigDecimal::from(0)
}

fn round_money(value: &BigDecimal) -> BigDecimal {
    value.round(2)
}

fn positive(value: &BigDecimal) -> BigDecimal {
    if value < &zero() {
        zero()
    } else {
        value.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxBand {
    pub lower_bound: BigDecimal,
    pub upper_bound: Option<BigDecimal>,
    pub base_tax: BigDecimal,
    pub marginal_rate: BigDecimal,
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
    Manual,
    External,
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
    InvalidTaxResidency,
    InvalidComponentAmount(String),
    InvalidComponentRate(String),
    DuplicateComponent(String),
    NegativeNetPay,
}

impl std::fmt::Display for PayrollCalculationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PayrollCalculationError {}

fn inclusive_days(start: NaiveDate, end: NaiveDate) -> i64 {
    (end - start).num_days() + 1
}

/// Prorates a salary using calendar days in the payroll period.
/// The overlap is inclusive at both ends.
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

    if let Some(end) = contract_end {
        if end < contract_start {
            return Err(PayrollCalculationError::InvalidContractPeriod);
        }
    }

    if salary < &zero() {
        return Err(PayrollCalculationError::NegativeBasicSalary);
    }

    let effective_start = period_start.max(contract_start);
    let effective_end = contract_end
        .map(|end| period_end.min(end))
        .unwrap_or(period_end);

    if effective_end < effective_start {
        return Ok(zero());
    }

    let period_days = inclusive_days(period_start, period_end);
    let eligible_days = inclusive_days(effective_start, effective_end);

    Ok(round_money(
        &(salary * BigDecimal::from(eligible_days)
            / BigDecimal::from(period_days)),
    ))
}

/// Progressive tax calculation using the configured band table.
/// base_tax is the cumulative tax payable at lower_bound.
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

    let mut tax = zero();

    for band in &ordered {
        if taxable_income <= &band.lower_bound {
            continue;
        }

        let taxable_upper = match &band.upper_bound {
            Some(upper) if taxable_income > upper => upper.clone(),
            Some(upper) => taxable_income.clone().min(upper.clone()),
            None => taxable_income.clone(),
        };

        let taxable_in_band = positive(&(taxable_upper - &band.lower_bound));

        tax += &band.base_tax
            + (taxable_in_band * &band.marginal_rate);

        // The next band contains the cumulative base tax.
        // Do not add the previous band's tax a second time.
        break;
    }

    // Find the highest band whose lower bound is not greater than income.
    let applicable = ordered
        .iter()
        .filter(|b| taxable_income >= &b.lower_bound)
        .last()
        .ok_or(PayrollCalculationError::MissingTaxBands)?;

    let taxable_in_band = match &applicable.upper_bound {
        Some(upper) => taxable_income.clone().min(upper.clone())
            - &applicable.lower_bound,
        None => taxable_income - &applicable.lower_bound,
    };

    Ok(round_money(
        &(&applicable.base_tax
            + positive(&taxable_in_band) * &applicable.marginal_rate),
    ))
}

fn component_amount(
    component: &ComponentInput,
    basic: &BigDecimal,
    gross_before_component: &BigDecimal,
) -> Result<BigDecimal, PayrollCalculationError> {
    if component.amount < zero() {
        return Err(PayrollCalculationError::InvalidComponentAmount(
            component.code.clone(),
        ));
    }

    let calculated = match component.method {
        CalculationMethod::Fixed | CalculationMethod::Manual => {
            component.amount.clone()
        }
        CalculationMethod::PercentOfBasic => {
            let rate = component.rate.as_ref().ok_or_else(|| {
                PayrollCalculationError::InvalidComponentRate(component.code.clone())
            })?;
            basic * rate
        }
        CalculationMethod::PercentOfGross => {
            let rate = component.rate.as_ref().ok_or_else(|| {
                PayrollCalculationError::InvalidComponentRate(component.code.clone())
            })?;
            gross_before_component * rate
        }
        CalculationMethod::External => {
            component.amount.clone()
        }
    };

    Ok(round_money(&calculated))
}

fn nssf_is_applicable(
    profile: &EmployeeTaxProfile,
    period_end: NaiveDate,
    minimum_age: Option<u8>,
    maximum_age: Option<u8>,
) -> bool {
    if !profile.nssf_covered || profile.nssf_exempt {
        return false;
    }

    let Some(dob) = profile.date_of_birth else {
        // Missing date of birth must be surfaced as a data-quality issue
        // by the caller; do not silently infer eligibility.
        return false;
    };

    let mut age = period_end.year() - dob.year();
    if (period_end.month(), period_end.day()) < (dob.month(), dob.day()) {
        age -= 1;
    }

    if let Some(minimum) = minimum_age {
        if age < i32::from(minimum) {
            return false;
        }
    }

    if let Some(maximum) = maximum_age {
        if age > i32::from(maximum) {
            return false;
        }
    }

    true
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

    let basic = prorate_salary(
        &input.basic_salary,
        input.period_start,
        input.period_end,
        input.contract_start,
        input.contract_end,
    )?;

    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut gross = basic.clone();
    let mut taxable = basic.clone();
    let mut nssf_base = basic.clone();
    let mut other_deductions = zero();
    let mut employer_costs = zero();

    for component in &input.components {
        if !seen.insert(component.code.to_uppercase()) {
            return Err(PayrollCalculationError::DuplicateComponent(
                component.code.clone(),
            ));
        }

        let basis = match component.method {
            CalculationMethod::PercentOfBasic => basic.clone(),
            CalculationMethod::PercentOfGross => gross.clone(),
            _ => component.amount.clone(),
        };

        let amount = component_amount(component, &basic, &gross)?;

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
                other_deductions += &amount;
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
    );

    let employee_nssf = if nssf_applicable {
        round_money(&(&nssf_base * &input.nssf_employee_rate))
    } else {
        zero()
    };

    let employer_nssf = if nssf_applicable {
        round_money(&(&nssf_base * &input.nssf_employer_rate))
    } else {
        zero()
    };

    let net = round_money(
        &(&gross - &paye - &employee_nssf - &other_deductions),
    );

    if net < zero() {
        return Err(PayrollCalculationError::NegativeNetPay);
    }

    let employer_cost_total = round_money(
        &(&gross + &employer_nssf + &employer_costs),
    );

    Ok(PayrollResult {
        employee_uuid: input.employee_uuid,
        basic_pay: basic,
        gross_pay: round_money(&gross),
        taxable_earnings: round_money(&taxable),
        paye,
        employee_nssf,
        employer_nssf,
        other_deductions: round_money(&other_deductions),
        employer_cost_total,
        net_pay: net,
        items,
    })
}
