import os
import re
from typing import Union, IO

SRC_DIR = "src"

first_cap_re = re.compile('(.)([A-Z][a-z]+)')
all_cap_re = re.compile('([a-z0-9])([A-Z])')


def camel_case_to_snake(name: str) -> str:
    s1 = first_cap_re.sub(r'\1_\2', name)
    return all_cap_re.sub(r'\1_\2', s1).lower()


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
                      "- Vec<X> means you need to put X objects in a [X, X, X] list.\n" + \
                      "- '=' sign after field means it is optional.\n"
        self.format += "\n"

        self.rename = "#[serde(rename = "
        self.to_rename = None
        self.defaulter = "#[serde(default ="
        self.to_default = None
        self.skipper = "#[serde(skip)]"
        self.to_skip = None
        self.FORCE_SKIP = "THIS TEXT FORCES SKIPPING"

    def add_line(self, line=""):
        self.format += line + "\n"

    def check_serde(self, line="") -> bool:
        serde = False
        if line.startswith(self.skipper):
            self.to_skip = True
            serde = True
        if line.startswith(self.rename):
            self.to_rename = line.split('"')[1]
            serde = True
        if line.startswith(self.defaulter):
            self.to_default = line.split('"')[1]
            serde = True
        return serde

    def create(self, file=None) -> str:
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

    def apply_serde(self, line: str) -> Union[str, None]:
        if self.to_rename is not None:
            line = self.to_rename + ":" + line.split(":")[1]
            self.to_rename = None
        if self.to_default is not None:
            assert line[-1] == ','
            line = line[:-1]
            line += " = " + self.to_default + ','
            self.to_default = None
        if self.to_skip is not None:
            self.to_skip = None
            return None
        return line

    def parse_enum(self):
        brace_count = 1
        for line in self.f:
            line = line.strip()
            if line.startswith("}"):
                brace_count -= 1
                if brace_count == 0:
                    self.add_line("}\n")
                    return
            if self.check_serde(line):
                continue
            line = self.apply_serde(line)
            if line:
                split = line.split(' ')
                name = camel_case_to_snake(split[0])
                line = ' '.join([name] + split[1:])
                self.add_line("    " * brace_count + line)
                if line.endswith("{"):
                    brace_count += 1

    def parse_struct(self):
        brace_count = 1
        for line in self.f:
            line = line.strip()
            if line.startswith("}"):
                brace_count -= 1
                if brace_count == 0:
                    self.add_line("}\n")
                    return
            if self.check_serde(line):
                continue
            if line:
                line = line[4:]
                line = self.apply_serde(line)
                if line is not None:
                    self.add_line("    " * brace_count + line)

            if line and line.endswith("{"):
                brace_count += 1


def game_components_file() -> IO:
    return open(os.path.join(SRC_DIR, "game_components.rs"), "r")


def format_file() -> IO:
    return open("format.txt", "w")


if __name__ == "__main__":
    with game_components_file() as f, format_file() as out:
        out.write(FormatCreator(f).create())
