import re
with open("src/mahjong/yaku.rs", "r") as f:
    text = f.read()

# Find all conflicts
conflicts = re.findall(r'<<<<<<< HEAD.*?=======(.*?)>>>>>>> origin/main', text, re.DOTALL)

for c in conflicts:
    # We want to keep the origin/main version of logic, but adapt it to our counts
    # Our counts are now passed in as `closed_counts`
    pass
