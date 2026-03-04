use csv::ReaderBuilder;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("service-names-port-numbers.csv")?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(BufReader::new(file));

    let mut assigned = BTreeSet::new();

    for result in rdr.records() {
        let record = result?;
        let service = record.get(0).unwrap_or("").trim();
        let port_str = record.get(1).unwrap_or("").trim();

        // Skip unassigned or empty
        if service.is_empty() || port_str == "Unassigned" {
            continue;
        }

        // Parse port range
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

    // Find gaps
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

    // Sort by size
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
