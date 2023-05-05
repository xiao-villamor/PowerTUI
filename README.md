# PowerTUI

A Simple terminal interface for interact with machines in your Vsphere. The program use the Vsphere API to interact with there.





## Installation

To run this program, you must have Rust and Cargo installed on your system. If you do not have Rust installed, you can install it by following the instructions on the Rust website.

Once Rust is installed, you can clone this repository to your local machine using the following command:

```bash
  git clone https://github.com/xiao-villamor/PowerTUI.git
```
    
## Usage/Examples

To run the program, navigate to the cloned repository directory and execute the following command:

```bash
cargo run
```

Before running the program, you should create a file called credentials.json in the project root directory. This file should contain the credentials needed to authenticate with the vSphere API. The format of the credentials.json file should be as follows:

```bash
{
  "ip": "192.168.1.100",
  "user": "your-username",
  "password": "your-password"
  "datacenter": "your-datacenter"
}
```



## Features
The program currently supports the following features:

- List all VMs in the inventory
- Power on a VM
- Power off a VM
- Reset a VM
- Suspend a VM


## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.



## Acknowledgements
This program was created as a demonstration for educational purposes and is not intended for production use.
