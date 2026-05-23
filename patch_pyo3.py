# Verify if WinContext is exposed to Python
with open("src/lib.rs", "r") as f:
    content = f.read()
    print("WinContext" in content)
