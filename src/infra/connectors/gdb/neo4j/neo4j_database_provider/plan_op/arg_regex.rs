use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub(super) static ref EXPAND: Regex =
        Regex::new(r"\((\w+)\)[<]{0,1}-\[(\w+)(:\w+){0,1}\]-[>]{0,1}\((\w+)\)").unwrap();
    pub(super) static ref DIRECTED_EDGE_TYPE: Regex =
        Regex::new(r"\((\w+)\)-\[(\w+):(\w+)\]->\((\w+)\)").unwrap();
    pub(super) static ref LABEL_PREDICATE: Regex = Regex::new(r"(\w+):(\w+)").unwrap();
    pub(super) static ref PROPERTY_PREDICATE: Regex = Regex::new(r"(\w+)[.](\w+)").unwrap();
    pub(super) static ref PROJECTION: Regex =
        Regex::new(r"(?P<expr>[^\s]+) AS (?:(?:`(?P<name1>.+)`)|(?P<name2>\w+))").unwrap();
    pub(super) static ref SIMPLE_EXPRESSION: Regex = Regex::new(
        r"(?:(?P<func>\w+)[\[\(])*(?P<name>[$]{0,1}\w+)(?:[.](?P<prop>\w+)){0,1}[\]\)]*"
    )
    .unwrap();
}
