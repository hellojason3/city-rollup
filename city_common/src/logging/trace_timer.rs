/*
NO_FORMAT="\x1b[0m"
F_BOLD="\x1b[1m"
C_LIGHTGOLDENROD1="\x1b[38;5;227m"
C_PURPLE="\x1b[48;5;93m"
echo -e "${F_BOLD}${C_LIGHTGOLDENROD1}${C_PURPLE}Color me, surprised${NO_FORMAT}"

NO_FORMAT="\x1b[0m"
C_GREENYELLOW="\x1b[48;5;154m"
echo -e "${C_GREENYELLOW}Violets are blue${NO_FORMAT}"

C_MEDIUMTURQUOISE="\x1b[38;5;80m"

C_WHEAT1="\x1b[38;5;229m"
C_DEEPSKYBLUE4="\x1b[48;5;23m"
*/
use std::time::Instant;

#[derive(Clone)]
pub struct TraceTimer {
    pub start_time: Instant,
    pub name: String,
}
impl TraceTimer {
    pub fn new(name: &str) -> Self {
        let n = name.to_string();
        Self {
            start_time: Instant::now(),
            name: n,
        }
    }
    pub fn lap(&mut self, event_name: &str) -> u64 {
        let elapsed = self.start_time.elapsed();
        let elapsed_ms = elapsed.as_millis() as u64;
        println!(
            "\x1b[1m\x1b[38;5;227m\x1b[48;5;93m{}\x1b[0m - \x1b[94m{}\x1b[0m: \x1b[38;5;229m\x1b[48;5;23m{}ms\x1b[0m",
            self.name, event_name, elapsed_ms
        );
        self.start_time = Instant::now();
        elapsed_ms
    }
    pub fn event(&mut self, event_name: String) -> u64 {
        let elapsed = self.start_time.elapsed();
        let elapsed_ms = elapsed.as_millis() as u64;
        println!(
            "\x1b[1m\x1b[38;5;227m\x1b[48;5;93m{}\x1b[0m - \x1b[94m{}\x1b[0m: \x1b[38;5;229m\x1b[48;5;23m{}ms\x1b[0m",
            self.name, event_name, elapsed_ms
        );
        self.start_time = Instant::now();
        elapsed_ms
    }
}
