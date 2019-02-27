import os

SRC_DIR = "src"


class FormatCreator:
    # TODO: add also the main game state format
    def __init__(self, file=None):
        self.f = file
        self.format = "- Struct means that all field must be included.\n" + \
                      "- Enum means that you choose one of the options.\n" + \
                      "- Even if something is written 'ExactlyLikeThis',\n" + \
                      "  you still have to write it 'exactly_like_this'.\n" + \
                      "- usize means that it must be a positive number.\n" + \
                      "- i32 means that it must be a number between -2^31 and 2^31.\n" + \
                      "- Vec<X> means you need to put X objects in a [X, X, X] list.\n"
        self.format += "\n"

    def add_line(self, line=""):
        self.format += line + "\n"

    def create(self, file=None):
        if file is not None:
            self.f = file
        for line in self.f:
            line = line.strip()
            if line.startswith("/// FORMAT END"):
                break
            line = line[4:]
            if line.startswith("struct"):
                self.add_line(line)
                self.parse_struct()
            if line.startswith("enum"):
                self.add_line(line)
                self.parse_enum()
            if line.startswith("type"):
                self.add_line(line + "\n")
        return self.format

    def parse_enum(self):
        for line in self.f:
            line = line.strip()
            if line.startswith("}"):
                self.add_line("}\n")
                return
            if line:
                self.add_line("    " + line)

    def parse_struct(self):
        rename = "#[serde(rename = "
        to_rename = None
        for line in self.f:
            line = line.strip()
            if line.startswith("}"):
                self.add_line("}\n")
                return
            if line.startswith("#[serde(skip)]"):
                next(self.f)
                continue
            if line.startswith(rename):
                to_rename = line.split('"')[1]
            if line.startswith("pub "):
                line = line[4:]
                if to_rename is not None:
                    line = to_rename + ":" + line.split(":")[1]
                    to_rename = None
                self.add_line("    " + line)


def game_components_file():
    return open(os.path.join(SRC_DIR, "game_components.rs"), "r")


def format_file():
    return open("format.txt", "w")


if __name__ == "__main__":
    with game_components_file() as f, format_file() as out:
        out.write(FormatCreator(f).create())
