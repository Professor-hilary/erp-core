//! Static seed data so e2e files stay readable.

pub struct Person {
    pub key: &'static str,
    pub first_name: &'static str,
    pub last_name: &'static str,
    pub email: &'static str,
    pub phone: &'static str,
    pub role: &'static str, // owner | shareholder | employee | lender_contact | ...
}

pub struct Institution {
    pub key: &'static str,
    pub name: &'static str,
    pub kind: &'static str, // bank | lender | supplier | customer | investor
    pub email: &'static str,
    pub tax_id: &'static str,
}

/// ≥25 individuals used across the suite
pub const PEOPLE: &[Person] = &[
    Person { key: "owner_hilary", first_name: "Hilary", last_name: "Okuonzi", email: "hilary.owner@test.local", phone: "+256700000001", role: "owner" },
    Person { key: "owner_sarah", first_name: "Sarah", last_name: "Nambi", email: "sarah.owner@test.local", phone: "+256700000002", role: "owner" },
    Person { key: "sh_james", first_name: "James", last_name: "Okello", email: "james.sh@test.local", phone: "+256700000003", role: "shareholder" },
    Person { key: "sh_grace", first_name: "Grace", last_name: "Achieng", email: "grace.sh@test.local", phone: "+256700000004", role: "shareholder" },
    Person { key: "sh_peter", first_name: "Peter", last_name: "Mugisha", email: "peter.sh@test.local", phone: "+256700000005", role: "shareholder" },
    Person { key: "emp_alice", first_name: "Alice", last_name: "Nakato", email: "alice.hr@test.local", phone: "+256700000006", role: "employee" },
    Person { key: "emp_bob", first_name: "Bob", last_name: "Ssekandi", email: "bob.prod@test.local", phone: "+256700000007", role: "employee" },
    Person { key: "emp_carol", first_name: "Carol", last_name: "Atuhaire", email: "carol.sales@test.local", phone: "+256700000008", role: "employee" },
    Person { key: "emp_dan", first_name: "Daniel", last_name: "Byaruhanga", email: "dan.warehouse@test.local", phone: "+256700000009", role: "employee" },
    Person { key: "emp_eva", first_name: "Eva", last_name: "Tumusiime", email: "eva.finance@test.local", phone: "+256700000010", role: "employee" },
    Person { key: "emp_frank", first_name: "Frank", last_name: "Kato", email: "frank.mfg@test.local", phone: "+256700000011", role: "employee" },
    Person { key: "emp_gina", first_name: "Gina", last_name: "Nabirye", email: "gina.qc@test.local", phone: "+256700000012", role: "employee" },
    Person { key: "emp_henry", first_name: "Henry", last_name: "Ouma", email: "henry.driver@test.local", phone: "+256700000013", role: "employee" },
    Person { key: "emp_irene", first_name: "Irene", last_name: "Namukasa", email: "irene.admin@test.local", phone: "+256700000014", role: "employee" },
    Person { key: "emp_john", first_name: "John", last_name: "Wasswa", email: "john.tech@test.local", phone: "+256700000015", role: "employee" },
    Person { key: "lend_mary", first_name: "Mary", last_name: "Akello", email: "mary.stanbic@test.local", phone: "+256700000016", role: "lender_contact" },
    Person { key: "lend_noah", first_name: "Noah", last_name: "Okot", email: "noah.centenary@test.local", phone: "+256700000017", role: "lender_contact" },
    Person { key: "vend_contact_1", first_name: "Paul", last_name: "Mugume", email: "paul@agrisupply.test", phone: "+256700000018", role: "vendor_contact" },
    Person { key: "vend_contact_2", first_name: "Quinn", last_name: "Nalubega", email: "quinn@packpro.test", phone: "+256700000019", role: "vendor_contact" },
    Person { key: "cust_contact_1", first_name: "Rita", last_name: "Kabonesa", email: "rita@freshmart.test", phone: "+256700000020", role: "customer_contact" },
    Person { key: "cust_contact_2", first_name: "Sam", last_name: "Lwanga", email: "sam@citygrocer.test", phone: "+256700000021", role: "customer_contact" },
    Person { key: "cust_contact_3", first_name: "Tina", last_name: "Nankya", email: "tina@hotelpearl.test", phone: "+256700000022", role: "customer_contact" },
    Person { key: "auditor", first_name: "Umar", last_name: "Ssali", email: "umar.audit@test.local", phone: "+256700000023", role: "advisor" },
    Person { key: "lawyer", first_name: "Vera", last_name: "Nakamya", email: "vera.legal@test.local", phone: "+256700000024", role: "advisor" },
    Person { key: "consultant", first_name: "Walter", last_name: "Opio", email: "walter.consult@test.local", phone: "+256700000025", role: "advisor" },
    Person { key: "board_chair", first_name: "Yvonne", last_name: "Among", email: "yvonne.board@test.local", phone: "+256700000026", role: "board" },
];

/// Institutions (banks, lenders, suppliers, customers, investors)
pub const INSTITUTIONS: &[Institution] = &[
    Institution { key: "bank_stanbic", name: "Stanbic Bank Uganda", kind: "bank", email: "corporate@stanbic.test", tax_id: "TIN-STB-001" },
    Institution { key: "bank_centenary", name: "Centenary Bank", kind: "bank", email: "business@centenary.test", tax_id: "TIN-CEN-002" },
    Institution { key: "lender_dfcu", name: "DFCU Bank Limited", kind: "lender", email: "credit@dfcu.test", tax_id: "TIN-DFCU-003" },
    Institution { key: "lender_ugadev", name: "Uganda Development Bank", kind: "lender", email: "sme@udb.test", tax_id: "TIN-UDB-004" },
    Institution { key: "inv_pearl", name: "Pearl Capital Partners", kind: "investor", email: "deals@pearl.test", tax_id: "TIN-PCP-005" },
    Institution { key: "sup_agri", name: "AgriSupply Ltd", kind: "supplier", email: "orders@agrisupply.test", tax_id: "TIN-AGS-006" },
    Institution { key: "sup_pack", name: "PackPro Industries", kind: "supplier", email: "sales@packpro.test", tax_id: "TIN-PPI-007" },
    Institution { key: "sup_chem", name: "ChemTrade East Africa", kind: "supplier", email: "info@chemtrade.test", tax_id: "TIN-CTE-008" },
    Institution { key: "sup_logistics", name: "SwiftHaul Logistics", kind: "supplier", email: "ops@swifthaul.test", tax_id: "TIN-SHL-009" },
    Institution { key: "cust_freshmart", name: "FreshMart Supermarkets", kind: "customer", email: "procurement@freshmart.test", tax_id: "TIN-FM-010" },
    Institution { key: "cust_citygrocer", name: "City Grocer Chain", kind: "customer", email: "buyers@citygrocer.test", tax_id: "TIN-CG-011" },
    Institution { key: "cust_hotelpearl", name: "Hotel Pearl Kampala", kind: "customer", email: "purchasing@hotelpearl.test", tax_id: "TIN-HP-012" },
    Institution { key: "cust_school", name: "Greenfield Secondary School", kind: "customer", email: "admin@greenfield.test", tax_id: "TIN-GSS-013" },
    Institution { key: "cust_hospital", name: "Nakasero Hospital", kind: "customer", email: "stores@nakasero.test", tax_id: "TIN-NH-014" },
    Institution { key: "cust_export", name: "East Africa Export Co", kind: "customer", email: "trade@eaexport.test", tax_id: "TIN-EAE-015" },
];