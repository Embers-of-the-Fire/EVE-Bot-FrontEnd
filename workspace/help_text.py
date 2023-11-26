import yaml
from PIL import Image, ImageDraw, ImageFont
from typing import *
from monad_std import Option
from monad_std.iter import IterMeta


class HelperNode(TypedDict):
    title: str
    desc: str


Helper: TypeAlias = HelperNode | Dict[str, "Helper"]


with open("./workspace/help_text.yaml", "r", encoding="utf-8") as f:
    help_text: Helper = yaml.load(f, yaml.CLoader)


def mapping(k: str, v: Helper, indent: int) -> List[Tuple[str, Option[Tuple[str, str]], int]]:
    if "title" in v.keys() and "desc" in v.keys():
        return [(k, Option.some((v["title"], v["desc"])), indent)]
    else:
        return (
            IterMeta.once((k, Option.none(), indent))
            .chain(IterMeta.iter(resolve(v, indent)))
            .collect_list()
        )


def resolve(val: Helper, indent: int) -> List[Tuple[str, Option[Tuple[str, str]], int]]:
    return IterMeta.iter(val.items()).map(lambda x: mapping(x[0], x[1], indent + 1)).flatten().collect_list()


for text, desc, indent in resolve(help_text['eve'], 0):
    indent -= 1
    indent_text = (indent * 8) * " "
    if desc.is_some():
        tag = ":" + text
    else:
        tag = "/" + text
    print(f'{indent_text + tag:<30}{desc.map(lambda x: f"{x[0]:<30}{x[1]}").unwrap_or("")}')
