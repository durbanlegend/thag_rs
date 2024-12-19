# Dependency Analysis Tools

## Setup

1. Create virtual environment: `python3 -m venv venv`
2. Activate: `source venv/bin/activate`
3. Install requirements: `pip install pandas matplotlib`

## Usage
### From tools/deps directory:

- Analyze dependencies: `./analyze_deps.sh`
- Analyze sizes: `./analyze_dep_sizes.sh`
- Track changes: `./track_deps.sh`
- Visualize trends: `python visualize_deps.py`

Starting again with simpler tools:
./analyze_single_crate.sh thag_core

## From tools/deps directory:

./analyze_deps.sh  # Analyze specific crate
./analyze_dep_sizes.sh ../../thag_core       # Analyze sizes for specific crate
./track_deps.sh                           # Track all crates
python visualize_deps.py                  # Generate graphs from tracking data
