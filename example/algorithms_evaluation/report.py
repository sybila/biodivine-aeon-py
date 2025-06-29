import glob
from itertools import zip_longest

script_names = [
    "fixed_points",
    "reachability_fwd",
    "reachability_bwd",
    "minimal_trap_spaces",
    "percolation",
]

with open("report.txt", "w") as report:
    for script_name in script_names:
        target_filename = f"**/edition-2022-aeon_{script_name}.py_times.csv"
        target_filename_new = f"**/edition-2022-aeon_{script_name}_new.py_times.csv"

        filepath = glob.glob(target_filename, recursive=True)[0]
        filepath_new = glob.glob(target_filename_new, recursive=True)[0]

        report.write(f"Comparing {script_name}.py with {script_name}_new.py:\n")

        try:
            with open(filepath, "r") as f1, open(filepath_new, "r") as f2:
                old_faster, new_faster = 0, 0
                percentage_speed = 0
                for line1, line2 in zip_longest(f1, f2, fillvalue=""):
                    if line1 == line2:
                        continue

                    model_number, time, *rest = line1.split(",")
                    model_number_new, time_new, *rest = line2.split(",")

                    if time.strip() == "FAIL":
                        report.write(
                            f"- {model_number}: Failed for the old implementation but succeeded for the new one.\n"
                        )
                        continue

                    if time_new.strip() == "FAIL":
                        report.write(
                            f"- {model_number}: Succeeded for the old implementation but failed for the new one.\n"
                        )
                        continue

                    time, time_new = float(time), float(time_new)

                    if time < time_new:
                        old_faster += 1
                    else:
                        new_faster += 1

                    percentage_speed += time_new / time

                    output = rest[0] if rest else "Empty"
                    output_new = rest[0] if rest else "Empty"
                    if output != output_new:
                        report.write(
                            f"- {model_number}: Different outputs for the old ({output}) and new ({output_new}) implementations.\n"
                        )

                report.write(
                    f"Results: {old_faster} models faster for the old implementation, {new_faster} models faster for the new implementation.\n"
                )
                report.write(
                    f"The new time was on average {percentage_speed / (old_faster + new_faster) * 100:.2f}% of the old time.\n"
                )

        except FileNotFoundError as e:
            report.write(f"Error: {e}\n")

        report.write("\n")
