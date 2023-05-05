//import the required libraries
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VapiVm {
    pub memory_size_MiB : u32,
    pub vm : String,
    pub name : String,
    pub power_state : String,
    pub cpu_count : u32,
}

impl VapiVm{
    fn new(memory_size_MiB : u32, vm : String, name : String, power_state : String, cpu_count : u32) -> VapiVm {
        VapiVm {
            memory_size_MiB,
            vm,
            name,
            power_state,
            cpu_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VapiHost {
    pub host : String,
    pub name : String,
    pub connection_state : String,
    pub power_state : String,
}

impl VapiHost   {
    fn new(host : String, name : String, connection_state : String, power_state : String) -> VapiHost {
        VapiHost {
            host,
            name,
            connection_state,
            power_state,
        }
    }
}
