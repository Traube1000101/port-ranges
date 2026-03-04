use csv::ReaderBuilder;
use reqwest::blocking::Client;
use std::collections::BTreeSet;

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (admin@heizu.dev)"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", APP_USER_AGENT);

    let client = Client::builder().user_agent(APP_USER_AGENT).build()?;

    let response = client
    .get("https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.csv")
    .send()?
    .bytes()?;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(response.as_ref());

    let mut assigned = BTreeSet::new();

    for result in reader.records() {
        let record = result?;
        let service = record.get(0).unwrap_or("").trim();
        let port_str = record.get(1).unwrap_or("").trim();

        if service.is_empty() || port_str == "Unassigned" {
            continue;
        }

        if let Ok(port) = port_str.parse::<u16>() {
            assigned.insert(port);
        } else if let Some(pos) = port_str.find('-') {
            if let (Ok(start), Ok(end)) = (
                port_str[..pos].trim().parse::<u16>(),
                port_str[pos + 1..].trim().parse::<u16>(),
            ) {
                for p in start..=end {
                    assigned.insert(p);
                }
            }
        }
    }

    let mut gaps = Vec::new();
    let mut prev = 0u16;

    for &port in assigned.iter() {
        if port > prev + 1 {
            gaps.push((prev + 1, port - 1, (port - prev - 1) as usize));
        }
        prev = port;
    }
    if prev < 65535 {
        gaps.push((prev + 1, 65535, (65535 - prev) as usize));
    }

    gaps.sort_by(|a, b| b.2.cmp(&a.2));

    println!(
        "Top 20 Free Port Ranges:\n\n{:5}\t{:5}\t{:5}",
        "Total", "Start", "End"
    );
    for (start, end, size) in gaps.iter().take(20) {
        println!("{:5}\t{:5}\t{:5}", size, start, end);
    }

    Ok(())
}
