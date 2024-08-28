use chrono::Timelike;
use chrono::Utc;
use std::collections::HashMap;
use std::net::SocketAddr;

//Todo: should be in config or env
//Todo: should make it bigger for production
pub const DDOS_THRESHOLD: usize = 100;

/*
    Use this struct to prevent DDoS attacks
    Hashmap is used to store the number of requests received from an address in the last 30 minutes
    Use two hashmaps to make sure that the hashmap is not cleared while it is being used
*/
pub struct DDoSPreventer {
    hashmap1: HashMap<SocketAddr, usize>,
    hashmap2: HashMap<SocketAddr, usize>,
    hashmap1_count: usize,
    hashmap2_count: usize,
}

impl DDoSPreventer {
    pub fn new() -> Self {
        DDoSPreventer {
            hashmap1: HashMap::new(),
            hashmap2: HashMap::new(),
            hashmap1_count: 0,
            hashmap2_count: 0,
        }
    }
    /*
    /// Check if the address should be allowed
    ///
    /// This function checks if the address should be allowed based on the number of requests
    /// received in the last 30 minute. If the number of requests exceeds the threshold, the address
    /// is not allowed.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to check
    ///
    /// # Returns
    ///
    /// * `true` if the address should be allowed, `false` otherwise
     *
     */
    pub fn should_allow(&mut self, addr: SocketAddr) -> bool {
        // get current minute
        let minutes = Utc::now().minute();
        let is_first_half = minutes < 30;
        // get current map and alternate map
        let (current_map, alternate_map) = if is_first_half {
            (&mut self.hashmap1, &mut self.hashmap2)
        } else {
            (&mut self.hashmap2, &mut self.hashmap1)
        };

        // if the address is found in the current map, increment the count
        let count = current_map.entry(addr).or_insert(0);
        *count += 1;

        // check if the count exceeds the threshold
        if *count > DDOS_THRESHOLD {
            return false;
        }

        // update the current map's counter
        if is_first_half {
            self.hashmap1_count += 1;
        } else {
            self.hashmap2_count += 1;
        }

        // check if the other map needs to be cleared
        if is_first_half && self.hashmap2_count > 0 {
            alternate_map.clear();
            self.hashmap2_count = 0;
        } else if !is_first_half && self.hashmap1_count > 0 {
            alternate_map.clear();
            self.hashmap1_count = 0;
        }
        true
    }
}
