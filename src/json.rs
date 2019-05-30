pub enum JsonValue {
    Null,
    Number(f64),
    Boolean(bool),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

//Need to write own iterate because none of the ones included in Rust is usable
struct StringIterator {
    characters: Vec<char>,
    position: usize,
}

impl StringIterator {
    fn new(chars: &str) -> StringIterator {
        StringIterator {
            characters: chars.chars().collect(),
            position: 0,
        }
    }

    fn peek(&self) -> Option<&char> {
        if self.position >= self.characters.len() {
            return None;
        }

        Some(&self.characters[self.position])
    }

    fn next(&mut self) -> Option<char> {
        if self.position >= self.characters.len() {
            return None;
        }

        let result = self.characters[self.position];
        self.position = self.position + 1;
        Some(result)
    }
}

pub fn parse_json(json: &str) -> Option<JsonValue> {
    let mut chars = StringIterator::new(json);
    skip_white_spaces(&mut chars);
    read_object(&mut chars)
}

fn skip_white_spaces(chars: &mut StringIterator) {
    while chars.peek().is_some() && is_white_space(chars.peek().unwrap()) {
        chars.next();
    }
}

fn is_white_space(c: &char) -> bool {
    (*c == ' ') || (*c == '\n') || (*c == '\r') || (*c == '\t')
}

fn read_bool(chars: &mut StringIterator) -> Option<JsonValue> {
    if chars.peek().is_some() {
        let c = *(chars.peek().unwrap());

        if c == 't' || c == 'T' {
            chars.next();
            chars.next();
            chars.next();
            chars.next();
            return Some(JsonValue::Boolean(true));
        } else if c == 'f' || c == 'F' {
            chars.next();
            chars.next();
            chars.next();
            chars.next();
            chars.next();
            return Some(JsonValue::Boolean(false));
        }
    }

    None
}

fn read_number(chars: &mut StringIterator) -> Option<JsonValue> {
    let mut number = String::new();
    while chars.peek().is_some()
        && (is_number_char(chars.peek().unwrap()) || *chars.peek().unwrap() == '.')
    {
        number.push(chars.next().unwrap());
    }

    if number.len() > 0 {
        let result: f64 = number.parse().unwrap();
        return Some(JsonValue::Number(result));
    }

    None
}

fn read_null(chars: &mut StringIterator) -> Option<JsonValue> {
    if chars.peek().is_some() {
        let c = *(chars.peek().unwrap());
        if c == 'N' || c == 'n' {
            chars.next();
            chars.next();
            chars.next();
            chars.next();
            return Some(JsonValue::Null);
        }
    }

    None
}

fn read_string(chars: &mut StringIterator) -> Option<JsonValue> {
    //Skip starting "
    chars.next();

    let mut result = String::new();
    while chars.peek().is_some() && *chars.peek().unwrap() != '"' {
        result.push(chars.next().unwrap());
    }
    //Skip trailing "
    chars.next();

    Some(JsonValue::String(result))
}

fn read_value(chars: &mut StringIterator) -> Option<JsonValue> {
    if chars.peek().is_some() {
        if is_bool_char(chars.peek().unwrap()) {
            return read_bool(chars);
        } else if is_null_char(chars.peek().unwrap()) {
            return read_null(chars);
        } else if is_number_char(chars.peek().unwrap()) {
            return read_number(chars);
        } else if is_string_char(chars.peek().unwrap()) {
            return read_string(chars);
        } else if is_array_char(chars.peek().unwrap()) {
            return read_array(chars);
        } else if is_object_char(chars.peek().unwrap()) {
            return read_object(chars);
        } else {
            let mut message = String::new();
            message.push_str("Unexpected character found: '");
            message.push(*chars.peek().unwrap());
            message.push_str("'. Expected JSON value start character.");
            panic!(message);
        }
    }

    None
}

fn read_array(chars: &mut StringIterator) -> Option<JsonValue> {
    //Skip leading [
    chars.next();

    let mut values = Vec::new();

    while chars.peek().is_some() && *chars.peek().unwrap() != ']' {
        skip_white_spaces(chars);
        let value = read_value(chars);

        if value.is_some() {
            values.push(value.unwrap());

            skip_white_spaces(chars);

            if chars.peek().is_some() {
                let is_comma = *chars.peek().unwrap() == ',';
                let is_array_end = *chars.peek().unwrap() == ']';
                if !is_comma && !is_array_end {
                    let mut message = String::new();
                    message.push_str("Expected , or ] after array value but found: ");
                    message.push(chars.next().unwrap());
                    panic!(message);
                }

                //Skip , for next value
                if is_comma {
                    chars.next();
                }
            } else {
                panic!("Unexpected EOF in array value!");
            }
        } else {
            panic!("Unexpected EOF in array value!");
        }
    }

    //Skip trailing ] at the end of array
    if *chars.peek().unwrap() == ']' {
        chars.next();
    } else {
        panic!("Unexpected EOF in array value!");
    }

    Some(JsonValue::Array(values))
}

fn read_object(chars: &mut StringIterator) -> Option<JsonValue> {
    //Skip leading {
    chars.next();

    let mut values = Vec::new();

    while chars.peek().is_some() && *chars.peek().unwrap() != '}' {
        skip_white_spaces(chars);

        let name_val = read_string(chars);
        let mut name = String::new();
        if let Some(JsonValue::String(n)) = name_val {
            name.push_str(n.as_str());
        } else {
            panic!("Could not read name for object field");
        }

        skip_white_spaces(chars);

        if chars.peek().is_some() && *chars.peek().unwrap() != ':' {
            let mut message = String::new();
            message.push_str("Expected : after object field name: ->");
            message.push_str(chars.position.to_string().as_str());
            message.push_str("<-");
            panic!(message);
        }

        //Skip :
        chars.next();

        skip_white_spaces(chars);
        let value = read_value(chars);

        if value.is_some() {
            values.push((name, value.unwrap()));

            skip_white_spaces(chars);

            if chars.peek().is_some() {
                let is_comma = *chars.peek().unwrap() == ',';
                let is_object_end = *chars.peek().unwrap() == '}';
                if !is_comma && !is_object_end {
                    let mut message = String::new();
                    message.push_str("Expected , or } after object field value but found: ");
                    message.push(chars.next().unwrap());
                    panic!(message);
                }

                //Skip , for next field
                if is_comma {
                    chars.next();
                }
            } else {
                panic!("Unexpected EOF in object value!");
            }
        } else {
            panic!("Unexpected EOF in object value!");
        }
    }

    //Skip trailing } at the end of array
    if *chars.peek().unwrap() == '}' {
        chars.next();
    } else {
        panic!("Unexpected EOF in object value!");
    }

    Some(JsonValue::Object(values))
}

fn is_bool_char(c: &char) -> bool {
    *c == 'F' || *c == 'f' || *c == 'T' || *c == 't'
}

fn is_null_char(c: &char) -> bool {
    *c == 'N' || *c == 'n'
}

fn is_string_char(c: &char) -> bool {
    *c == '"'
}

fn is_number_char(c: &char) -> bool {
    *c == '0'
        || *c == '1'
        || *c == '2'
        || *c == '3'
        || *c == '4'
        || *c == '5'
        || *c == '6'
        || *c == '7'
        || *c == '8'
        || *c == '9'
        || *c == '+'
        || *c == '-'
}

fn is_array_char(c: &char) -> bool {
    *c == '['
}

fn is_object_char(c: &char) -> bool {
    *c == '{'
}
