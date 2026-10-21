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
    Person { key: "owner_heather", first_name: "Heather", last_name: "Erica", email: "heather.owner@test.local", phone: "+256700000002", role: "owner" },
    Person { key: "owner_sarah", first_name: "Sarah", last_name: "Nambi", email: "sarah.owner@test.local", phone: "+256700000003", role: "owner" },
    Person { key: "sh_james", first_name: "James", last_name: "Okello", email: "james.sh@test.local", phone: "+256700000010", role: "shareholder" },
    Person { key: "sh_grace", first_name: "Grace", last_name: "Achieng", email: "grace.sh@test.local", phone: "+256700000011", role: "shareholder" },
    Person { key: "sh_peter", first_name: "Peter", last_name: "Mugisha", email: "peter.sh@test.local", phone: "+256700000012", role: "shareholder" },
    Person { key: "ceo_hilary", first_name: "Hilary", last_name: "Okuonzi", email: "hilary.mgt@test.local", phone: "+256700000020", role: "employee" },
    Person { key: "emp_alice", first_name: "Alice", last_name: "Nakato", email: "alice.hr@test.local", phone: "+256700000021", role: "employee" },
    Person { key: "emp_bob", first_name: "Bob", last_name: "Ssekandi", email: "bob.prod@test.local", phone: "+256700000022", role: "employee" },
    Person { key: "emp_carol", first_name: "Carol", last_name: "Atuhaire", email: "carol.sales@test.local", phone: "+256700000023", role: "employee" },
    Person { key: "emp_dan", first_name: "Daniel", last_name: "Byaruhanga", email: "dan.warehouse@test.local", phone: "+256700000024", role: "employee" },
    Person { key: "emp_eva", first_name: "Eva", last_name: "Tumusiime", email: "eva.finance@test.local", phone: "+256700000025", role: "employee" },
    Person { key: "emp_frank", first_name: "Frank", last_name: "Kato", email: "frank.mfg@test.local", phone: "+256700000026", role: "employee" },
    Person { key: "emp_gina", first_name: "Gina", last_name: "Nabirye", email: "gina.qc@test.local", phone: "+256700000027", role: "employee" },
    Person { key: "emp_henry", first_name: "Henry", last_name: "Ouma", email: "henry.driver@test.local", phone: "+256700000028", role: "employee" },
    Person { key: "emp_irene", first_name: "Irene", last_name: "Namukasa", email: "irene.admin@test.local", phone: "+256700000029", role: "employee" },
    Person { key: "emp_john", first_name: "John", last_name: "Wasswa", email: "john.tech@test.local", phone: "+256700000030", role: "employee" },
    Person { key: "lend_mary", first_name: "Mary", last_name: "Akello", email: "mary.stanbic@test.local", phone: "+256700000040", role: "lender_contact" },
    Person { key: "lend_noah", first_name: "Noah", last_name: "Okot", email: "noah.centenary@test.local", phone: "+256700000041", role: "lender_contact" },
    Person { key: "vend_contact_1", first_name: "Paul", last_name: "Mugume", email: "paul@agrisupply.test", phone: "+256700000050", role: "vendor_contact" },
    Person { key: "vend_contact_2", first_name: "Quinn", last_name: "Nalubega", email: "quinn@packpro.test", phone: "+256700000051", role: "vendor_contact" },
    Person { key: "cust_contact_1", first_name: "Rita", last_name: "Kabonesa", email: "rita@freshmart.test", phone: "+256700000060", role: "customer_contact" },
    Person { key: "cust_contact_2", first_name: "Sam", last_name: "Lwanga", email: "sam@citygrocer.test", phone: "+256700000061", role: "customer_contact" },
    Person { key: "cust_contact_3", first_name: "Tina", last_name: "Nankya", email: "tina@hotelpearl.test", phone: "+256700000062", role: "customer_contact" },
    Person { key: "auditor", first_name: "Umar", last_name: "Ssali", email: "umar.audit@test.local", phone: "+256700000070", role: "advisor" },
    Person { key: "lawyer", first_name: "Vera", last_name: "Nakamya", email: "vera.legal@test.local", phone: "+256700000071", role: "advisor" },
    Person { key: "consultant", first_name: "Walter", last_name: "Opio", email: "walter.consult@test.local", phone: "+256700000072", role: "advisor" },
    Person { key: "board_chair", first_name: "Yvonne", last_name: "Among", email: "yvonne.board@test.local", phone: "+256700000080", role: "board" },
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

	// suppliers
	Institution { key: "sup_mcu", name: "Nile Embedded Components", kind: "supplier", email: "orders@nile-embed.test", tax_id: "TIN-NEC-006" },	
	Institution { key: "sup_power", name: "Lake Battery Systems", kind: "supplier", email: "sales@lakebatt.test", tax_id: "TIN-LBS-007" },
	Institution { key: "sup_mech", name: "Kampala Precision Fab", kind: "supplier", email: "cnc@kpfab.test", tax_id: "TIN-KPF-008" },
	Institution { key: "sup_logistics", name: "SwiftHaul Logistics", kind: "supplier", email: "ops@swifthaul.test", tax_id: "TIN-SHL-009" },
	// customers (platform buyers)
	Institution { key: "cust_factory", name: "Namanve Industrial Park Ops", kind: "customer", email: "automation@namanve.test", tax_id: "TIN-NIP-010" },
	Institution { key: "cust_warehouse", name: "EastLog Fulfilment Hub", kind: "customer", email: "robotics@eastlog.test", tax_id: "TIN-ELF-011" },
	Institution { key: "cust_utility", name: "Uganda Grid Services", kind: "customer", email: "inspection@ugs.test", tax_id: "TIN-UGS-012" },
	Institution { key: "cust_marina", name: "Lake Victoria Marine Services", kind: "customer", email: "rov@lakevictoria.test", tax_id: "TIN-LVM-013" },
	Institution { key: "cust_farm", name: "Busoga Agri Cooperative", kind: "customer", email: "precision@busoga-agri.test", tax_id: "TIN-BAC-014" },
	Institution { key: "cust_mine", name: "Karamoja Survey Ltd", kind: "customer", email: "mapping@karamoja-survey.test", tax_id: "TIN-KSL-015" },
];

/// Robotics OEM catalog for e2e (CargoMule-first).
#[allow(dead_code)]
pub struct RmSpec {
    pub sku: &'static str,
    pub name: &'static str,
    pub unit: &'static str,
    pub standard_cost: &'static str,
    pub unit_price: &'static str, // purchase price
    pub qty_buy: &'static str,    // P2P buy qty
}

/// Leaves purchased in e2e_05
#[allow(dead_code)]
pub const RMS: &[RmSpec] = &[
    RmSpec { sku: "MCU-STM32", name: "STM32 MCU", unit: "pc", standard_cost: "45000", unit_price: "45000", qty_buy: "50" },
    RmSpec { sku: "MCU-ESP32", name: "ESP32 Module", unit: "pc", standard_cost: "28000", unit_price: "28000", qty_buy: "40" },
    RmSpec { sku: "IMU-9DOF", name: "9-DOF IMU", unit: "pc", standard_cost: "55000", unit_price: "55000", qty_buy: "30" },
    RmSpec { sku: "ESC-40A", name: "ESC 40A", unit: "pc", standard_cost: "35000", unit_price: "35000", qty_buy: "40" },
    RmSpec { sku: "MTR-BLDC", name: "BLDC Motor", unit: "pc", standard_cost: "42000", unit_price: "42000", qty_buy: "40" },
    RmSpec { sku: "BAT-4S5AH", name: "Battery 4S 5Ah", unit: "pc", standard_cost: "180000", unit_price: "180000", qty_buy: "20" },
    RmSpec { sku: "PCB-CTRL-V1", name: "Control PCB v1", unit: "pc", standard_cost: "25000", unit_price: "25000", qty_buy: "30" },
    RmSpec { sku: "FRM-GROUND", name: "Ground Robot Chassis", unit: "pc", standard_cost: "220000", unit_price: "220000", qty_buy: "10" },
    RmSpec { sku: "SENS-US", name: "Ultrasonic Sensor", unit: "pc", standard_cost: "12000", unit_price: "12000", qty_buy: "40" },
    RmSpec { sku: "CBL-HARN", name: "Wiring Harness", unit: "pc", standard_cost: "15000", unit_price: "15000", qty_buy: "30" },
    RmSpec { sku: "FST-KIT", name: "Fastener Kit", unit: "kit", standard_cost: "8000", unit_price: "8000", qty_buy: "50" },
];

#[allow(dead_code)]
pub const FG_CARGO_MULE: &str = "FG-CARGO-MULE";
#[allow(dead_code)]
pub const SA_CTRL_CORE: &str = "SA-CTRL-CORE";
#[allow(dead_code)]
pub const SA_PROP_GROUND: &str = "SA-PROP-GROUND";

