use std::io::{self, Write};

use super::update_outcome::UpdateOutcome;

pub fn write_outcome(output: &mut impl Write, outcome: &UpdateOutcome) -> io::Result<()> {
    match outcome {
        UpdateOutcome::UpToDate { current } => {
            writeln!(output, "zeldoc {current} is the latest version.")
        }
        UpdateOutcome::Available { current, latest } => writeln!(
            output,
            "zeldoc {latest} is available (installed: {current}). Run `zeldoc update` to install it."
        ),
        UpdateOutcome::Updated {
            previous,
            installed,
        } => writeln!(output, "Updated zeldoc from {previous} to {installed}."),
    }
}

#[cfg(test)]
mod tests {
    use axoupdater::Version;

    use super::*;

    fn version(text: &str) -> Version {
        text.parse().unwrap()
    }

    fn render(outcome: &UpdateOutcome) -> String {
        let mut output = Vec::new();
        write_outcome(&mut output, outcome).unwrap();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn reports_each_outcome() {
        assert_eq!(
            render(&UpdateOutcome::UpToDate {
                current: version("0.3.0")
            }),
            "zeldoc 0.3.0 is the latest version.\n"
        );
        assert_eq!(
            render(&UpdateOutcome::Available {
                current: version("0.2.0"),
                latest: version("0.3.0")
            }),
            "zeldoc 0.3.0 is available (installed: 0.2.0). Run `zeldoc update` to install it.\n"
        );
        assert_eq!(
            render(&UpdateOutcome::Updated {
                previous: version("0.2.0"),
                installed: version("0.3.0")
            }),
            "Updated zeldoc from 0.2.0 to 0.3.0.\n"
        );
    }
}
