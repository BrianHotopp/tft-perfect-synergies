# tft-perfect-synergies
Generate TFT team comps with few wasted traits. This program outputs JSON containing the team comps and metadata for each team. See [here](./examples/teams_sizes_0_to_3_max_waste_0_1669402544.json) for an example of the output format (contains all teams less than or equal to size 3 with no wasted traits).
## Using this tool
Click releases on the right and download the binary for your system currently only linux and windows binaries are provided, but you should be able to run this on OSX by compiling it yourself with `cargo`.
Run the program from the command line using `./<program_name> <output_folder> <max_waste> <min_teamsize> <max_teamsize> <champs_filename> <traits_filename>`
## Arguments
### output_folder
Folder the script will write the output file to.
### max_waste
The maximum number of "wasted traits" a team can have. A wasted trait is a trait that is not necessary for some synergy to be active. For example if the Bruiser trait has breakpoints 2, 4, and 6, and you have 3 bruisers on your team, then you have at least 1 wasted trait. If you have a team of 4 champions and each champion only has the bruiser trait, then you have a team with 0 wasted traits. Teams with 0 wasted traits are also called "perfect synergies." [This tool](https://tactics.tools/perfect-synergies) shows teams with 0-2 wasted traits. It doesn't show teams with more than 3 because there are far to many to reasonably put on a webpage, but it is worth searching the team comp space for teams with 3-4 wasted traits because these teams are often the most powerful.
### min_teamsize
The minimum number of champions a team can have.
### max_teamsize
The maximum number of champions a team can have.
### champs_filename
The name of the file containing the necessary champion data. An example of the input format can be seen [here](./examples/champs_1669402544.json). To automatically generate this file by scraping mobafire, see [this tool](https://github.com/BrianHotopp/tft-unit-data-scraper)
### traits_filename
The name of the file containing the necessary trait data. An example of the input format can be seen [here](./examples/traits_1669402544.json). To automatically generate this file by scraping mobafire, see [this tool](https://github.com/BrianHotopp/tft-unit-data-scraper)
## About
There are ~60 champions in TFT, so brute force searching through all 435,878,172,349 teams of size 0-9 can take a while. It takes around 3.5 h on my laptop's i7-8750h. The program checks the teams in parallel and doesn't try to load all the combinations into memory at once, so in theory it can handle teams of size 10 or 11, but since it checks roughly 2.3*10^6 combinations/second it will around 9 hours to check only the teams of size 10 (and longer for larger team sizes).
## Development/Contributing
If you can find a way to do this that is better than brute force, please submit a pr! Also if you are interested in searching for teams with other interesting properties, let me know! I would be happy to help you get started.
