use chrono::NaiveDate;

#[derive(Debug)]
pub struct OFXTransaction {
    pub trntype: Option<String>,
    pub fitid: Option<String>,
    pub name: Option<String>,
    pub memo: Option<String>,
    pub date: NaiveDate,
    pub amount_cents: i32,
}

#[derive(Debug)]
pub struct OFXStatement {
    pub transactions: Vec<OFXTransaction>,
    pub balance_cents: i32,
}

fn extract_tag(block: &str, tag: &str) -> Option<String> {
    let pattern = format!("<{}>([^<]*)", tag);
    let re = regex::Regex::new(&pattern).ok()?;
    let cap = re.captures(block)?;
    Some(cap[1].trim().to_string())
}

fn parse_ofx_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    let s = if s.contains('[') {
        s.split('[').next().unwrap_or(s)
    } else {
        s
    };

    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() >= 8 {
        let y: i32 = digits[0..4].parse().ok()?;
        let m: u32 = digits[4..6].parse().ok()?;
        let d: u32 = digits[6..8].parse().ok()?;
        NaiveDate::from_ymd_opt(y, m, d)
    } else {
        None
    }
}

fn parse_amount_cents(s: &str) -> Option<i32> {
    let s = s.trim().replace(',', "");
    let val: f64 = s.parse().ok()?;
    Some((val * 100.0).round() as i32)
}

pub fn parse_ofx(content: &str) -> Result<OFXStatement, String> {
    let balance = extract_tag(content, "BALAMT")
        .and_then(|s| parse_amount_cents(&s))
        .unwrap_or(0);

    let stmt_re = regex::Regex::new(r"(?s)<STMTTRN>(.*?)</STMTTRN>")
        .map_err(|e| format!("regex: {e}"))?;

    let mut transactions = Vec::new();

    for cap in stmt_re.captures_iter(content) {
        if let Some(block) = cap.get(1) {
            let block = block.as_str();
            let date_str = extract_tag(block, "DTPOSTED").unwrap_or_default();
            let date = parse_ofx_date(&date_str).unwrap_or_default();

            let amount_str = extract_tag(block, "TRNAMT").unwrap_or_default();
            let amount_cents = parse_amount_cents(&amount_str).unwrap_or(0);

            transactions.push(OFXTransaction {
                trntype: extract_tag(block, "TRNTYPE"),
                fitid: extract_tag(block, "FITID"),
                name: extract_tag(block, "NAME"),
                memo: extract_tag(block, "MEMO"),
                date,
                amount_cents,
            });
        }
    }

    transactions.sort_by(|a, b| a.fitid.cmp(&b.fitid));

    Ok(OFXStatement {
        transactions,
        balance_cents: balance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OFX: &str = r#"
OFXHEADER:100
DATA:OFXSGML
VERSION:230

<OFX>
<BANKACCTFROM>
<BANKID>123456789
<ACCTID>123456
</BANKACCTFROM>

<BANKTRANLIST>
<STMTTRN>
<TRNTYPE>CREDIT
<DTPOSTED>20240115
<TRNAMT>1234.56
<FITID>TXN001
<NAME>Paycheck
</STMTTRN>
<STMTTRN>
<TRNTYPE>DEBIT
<DTPOSTED>20240116
<TRNAMT>-50.25
<FITID>TXN002
<NAME>Grocery Store
</STMTTRN>
</BANKTRANLIST>

<LEDGERBAL>
<BALAMT>5678.90
</LEDGERBAL>
</OFX>
"#;

    #[test]
    fn test_parse_ofx() {
        let result = parse_ofx(SAMPLE_OFX).unwrap();
        assert_eq!(result.balance_cents, 567890);
        assert_eq!(result.transactions.len(), 2);
        assert_eq!(result.transactions[0].fitid.as_deref(), Some("TXN001"));
        assert_eq!(result.transactions[0].amount_cents, 123456);
        assert_eq!(result.transactions[0].name.as_deref(), Some("Paycheck"));
        assert_eq!(result.transactions[1].amount_cents, -5025);
    }
}
