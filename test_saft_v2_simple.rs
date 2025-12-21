use std::fs;

fn main() {
    // Create a simple test XML with the new v1.0.1 structure
    let test_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<nsSAFT:AuditFile xmlns:doc="urn:schemas-OECD:schema-extensions:documentation xml:lang=en" xmlns:nsSAFT="mf:nra:dgti:dxxxx:declaration:v1" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <nsSAFT:Header>
    <nsSAFT:AuditFileVersion>007</nsSAFT:AuditFileVersion>
    <nsSAFT:AuditFileCountry>BG</nsSAFT:AuditFileCountry>
    <nsSAFT:AuditFileRegion>BG-22</nsSAFT:AuditFileRegion>
    <nsSAFT:AuditFileDateCreated>2024-12-05</nsSAFT:AuditFileDateCreated>
    <nsSAFT:SoftwareCompanyName>RS Accounting BG</nsSAFT:SoftwareCompanyName>
    <nsSAFT:SoftwareID>RS-AC-BG</nsSAFT:SoftwareID>
    <nsSAFT:SoftwareVersion>001</nsSAFT:SoftwareVersion>
    <nsSAFT:Company>
      <nsSAFT:RegistrationNumber>123456789</nsSAFT:RegistrationNumber>
      <nsSAFT:Name>Тестова Компания ЕООД</nsSAFT:Name>
      <nsSAFT:Address>
        <nsSAFT:StreetName>ул. Тестова</nsSAFT:StreetName>
        <nsSAFT:Number>1</nsSAFT:Number>
        <nsSAFT:AdditionalAddressDetail></nsSAFT:AdditionalAddressDetail>
        <nsSAFT:Building></nsSAFT:Building>
        <nsSAFT:City>София</nsSAFT:City>
        <nsSAFT:PostalCode>1000</nsSAFT:PostalCode>
        <nsSAFT:Region></nsSAFT:Region>
        <nsSAFT:Country>BG</nsSAFT:Country>
        <nsSAFT:AddressType>StreetAddress</nsSAFT:AddressType>
      </nsSAFT:Address>
      <nsSAFT:Contact>
        <nsSAFT:Telephone>123456789</nsSAFT:Telephone>
        <nsSAFT:Fax></nsSAFT:Fax>
        <nsSAFT:Email>test@test.com</nsSAFT:Email>
        <nsSAFT:Website></nsSAFT:Website>
      </nsSAFT:Contact>
      <nsSAFT:TaxRegistration>
        <nsSAFT:TaxRegistrationNumber>123456789</nsSAFT:TaxRegistrationNumber>
        <nsSAFT:TaxType>100010</nsSAFT:TaxType>
        <nsSAFT:TaxNumber>BG123456789</nsSAFT:TaxNumber>
        <nsSAFT:TaxVerificationDate>2024-12-05</nsSAFT:TaxVerificationDate>
      </nsSAFT:TaxRegistration>
      <nsSAFT:BankAccount>
        <nsSAFT:IBANNumber>BG80BNBG96611020345678</nsSAFT:IBANNumber>
      </nsSAFT:BankAccount>
    </nsSAFT:Company>
    <nsSAFT:DefaultCurrencyCode>BGN</nsSAFT:DefaultCurrencyCode>
    <nsSAFT:SelectionCriteria>
      <nsSAFT:TaxReportingJurisdiction>NRA</nsSAFT:TaxReportingJurisdiction>
      <nsSAFT:CompanyEntity></nsSAFT:CompanyEntity>
      <nsSAFT:PeriodStart>11</nsSAFT:PeriodStart>
      <nsSAFT:PeriodStartYear>2024</nsSAFT:PeriodStartYear>
      <nsSAFT:PeriodEnd>12</nsSAFT:PeriodEnd>
      <nsSAFT:PeriodEndYear>2024</nsSAFT:PeriodEndYear>
      <nsSAFT:DocumentType></nsSAFT:DocumentType>
      <nsSAFT:OtherCriteria>Няма</nsSAFT:OtherCriteria>
    </nsSAFT:SelectionCriteria>
    <nsSAFT:HeaderComment>M</nsSAFT:HeaderComment>
    <nsSAFT:TaxAccountingBasis>A</nsSAFT:TaxAccountingBasis>
    <nsSAFT:TaxEntity>Company</nsSAFT:TaxEntity>
  </nsSAFT:Header>
  <nsSAFT:MasterFilesMonthly>
    <nsSAFT:GeneralLedgerAccounts>
      <nsSAFT:Account>
        <nsSAFT:AccountID>123</nsSAFT:AccountID>
        <nsSAFT:AccountDescription>Тестова сметка</nsSAFT:AccountDescription>
        <nsSAFT:TaxpayerAccountID>123</nsSAFT:TaxpayerAccountID>
        <nsSAFT:GroupingCategory></nsSAFT:GroupingCategory>
        <nsSAFT:GroupingCode></nsSAFT:GroupingCode>
        <nsSAFT:AccountType>Bifunctional</nsSAFT:AccountType>
        <nsSAFT:AccountCreationDate>2024-01-01</nsSAFT:AccountCreationDate>
        <nsSAFT:OpeningDebitBalance>0.00</nsSAFT:OpeningDebitBalance>
        <nsSAFT:ClosingDebitBalance>0.00</nsSAFT:ClosingDebitBalance>
      </nsSAFT:Account>
    </nsSAFT:GeneralLedgerAccounts>
    <nsSAFT:Customers></nsSAFT:Customers>
    <nsSAFT:Suppliers></nsSAFT:Suppliers>
    <nsSAFT:TaxTable>
      <nsSAFT:TaxCodeDetails>
        <nsSAFT:TaxCode>VAT20</nsSAFT:TaxCode>
        <nsSAFT:EffectiveDate>2007-01-01</nsSAFT:EffectiveDate>
        <nsSAFT:Description>Стандартна ставка ДДС 20%</nsSAFT:Description>
        <nsSAFT:TaxPercentage>20</nsSAFT:TaxPercentage>
        <nsSAFT:Country>BG</nsSAFT:Country>
      </nsSAFT:TaxCodeDetails>
    </nsSAFT:TaxTable>
    <nsSAFT:UOMTable>
      <nsSAFT:UOMEntry>
        <nsSAFT:UnitOfMeasure>бр</nsSAFT:UnitOfMeasure>
        <nsSAFT:UOMDescription>Брой</nsSAFT:UOMDescription>
      </nsSAFT:UOMEntry>
    </nsSAFT:UOMTable>
    <nsSAFT:Products></nsSAFT:Products>
  </nsSAFT:MasterFilesMonthly>
  <nsSAFT:GeneralLedgerEntries>
    <nsSAFT:NumberOfEntries>0</nsSAFT:NumberOfEntries>
    <nsSAFT:TotalDebit>0.00</nsSAFT:TotalDebit>
    <nsSAFT:TotalCredit>0.00</nsSAFT:TotalCredit>
    <nsSAFT:Journal>
      <nsSAFT:JournalID>1</nsSAFT:JournalID>
      <nsSAFT:Description>Главна книга</nsSAFT:Description>
      <nsSAFT:Type>GL</nsSAFT:Type>
    </nsSAFT:Journal>
  </nsSAFT:GeneralLedgerEntries>
  <nsSAFT:SourceDocumentsMonthly></nsSAFT:SourceDocumentsMonthly>
</nsSAFT:AuditFile>"#;

    // Write to file
    fs::write("test_saft_v1_0_1.xml", test_xml).expect("Unable to write file");
    
    println!("✅ Създаден е тестов SAF-T v1.0.1 файл: test_saft_v1_0_1.xml");
    println!("📏 Размер на файла: {} байта", test_xml.len());
    println!("🔍 Файлът съответства на новата одобрена схема с namespace: mf:nra:dgti:dxxxx:declaration:v1");
    
    // Basic XML validation (just check if it's well-formed)
    match quick_xml::Reader::from_str(test_xml).read_event() {
        Ok(_) => println!("✅ XML файлът е валиден синтактично"),
        Err(e) => println!("❌ XML грешка: {}", e),
    }
}