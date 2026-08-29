//! The PrismPM command-line binary.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    prismpm::cli::run()
}

