import pandas as pd
import matplotlib.pyplot as plt
from datetime import datetime
import glob
import os

def plot_crate_history(log_file):
    if not os.path.exists(log_file):
        print(f"No history file found: {log_file}")
        return

    df = pd.read_csv(log_file)
    if df.empty:
        print(f"No data in {log_file}")
        return

    crate_name = os.path.basename(log_file).replace('_history.csv', '')

    df['Date'] = pd.to_datetime(df['Date'])

    plt.figure(figsize=(12, 6))
    plt.plot(df['Date'], df['Direct Dependencies'], label='Direct', marker='o')
    plt.plot(df['Date'], df['Indirect Dependencies'], label='Indirect', marker='o')

    plt.title(f'{crate_name} Dependencies Over Time')
    plt.xlabel('Date')
    plt.ylabel('Number of Dependencies')
    plt.legend()
    plt.grid(True)
    plt.xticks(rotation=45)
    plt.tight_layout()

    output_file = f'dependency_trend_{crate_name}.png'
    plt.savefig(output_file)
    print(f"Generated {output_file}")
    plt.close()

def main():
    log_dir = "dependency_logs"
    if not os.path.exists(log_dir):
        print("No dependency logs found. Run track_deps.sh first.")
        return

    log_files = glob.glob(f"{log_dir}/*_history.csv")
    if not log_files:
        print("No history files found in dependency_logs/")
        return

    for log_file in log_files:
        plot_crate_history(log_file)

if __name__ == "__main__":
    main()
