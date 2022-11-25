# tft-perfect-synergies
Generate TFT team comps with few wasted traits. Outputs JSON containing the team comps and metadata for each team. See [here] for an example of the output format (contains all teams less than or equal to size 4 with no wasted traits).
## Using this tool
Click releases and download the binary for your system (only linux and windows binaries are provided, but you should be able to run this on OSX by compiling it yourself with `cargo`)
Run the program from the command line using `./<program-name> <output folder> <max waste> <min teamsize> <max teamsize> <champs_filename> <traits_filename>`
## Arguments
### output folder
Folder the script will write the output file to.
### max waste
The maximum allowed number of "wasted traits" a team is allowed to have. For example 

## Building the tool
cargo build --release in the project root
## OSX Support
You can compile this code and use it on Mac. Clone the repository and run `cargo build --release` (requires you to have the `cargo` rust tool installed), then run the script located in `./target/release/`.
