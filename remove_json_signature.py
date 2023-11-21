import json
import os


def main():
    for file in os.listdir("./JSON-SDK"):
        if file.endswith(".json"):
            with open(f"./JSON-SDK/{file}", "r") as f:
                data = json.load(f)
                del data["DougTheDruid"]
            with open(f"./JSON-SDK/{file}", "w") as f:
                json.dump(data, f, indent=2)


if __name__ == "__main__":
    main()
