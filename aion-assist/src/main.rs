fn main() {
    match aion_assist::run() {
        Ok(aion_assist::Outcome::Done) => {}
        Ok(aion_assist::Outcome::Help(text)) => println!("{text}"),
        Err(err) => {
            eprintln!("aion-assist: {err}");
            std::process::exit(1);
        }
    }
}
