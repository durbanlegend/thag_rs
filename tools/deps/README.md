# Dependency Analysis Tools

## Setup
1. Create virtual environment: `python3 -m venv venv`
2. Activate: `source venv/bin/activate`
3. Install requirements: `pip install pandas matplotlib`

## Usage
- Analyze dependencies: `./analyze_deps.sh`
- Analyze sizes: `./analyze_dep_sizes.sh`
- Track changes: `./track_deps.sh`
- Visualize trends: `python visualize_deps.py`

# From tools/deps directory:
./analyze_deps.sh ../../thag_core thag_core  # Analyze specific crate
./analyze_dep_sizes.sh ../../thag_core       # Analyze sizes for specific crate
./track_deps.sh                           # Track all crates
python visualize_deps.py                  # Generate graphs from tracking data
