use super::Nl80211Iftype;

pub fn decode_iftypes(bytes: Vec<u8>) -> Vec<Nl80211Iftype> {
    bytes
        .chunks(4)
        .filter_map(|chunk| {
            if chunk.len() == 4 {
                match chunk[2] {
                    0 => Some(Nl80211Iftype::IftypeUnspecified),
                    1 => Some(Nl80211Iftype::IftypeAdhoc),
                    2 => Some(Nl80211Iftype::IftypeStation),
                    3 => Some(Nl80211Iftype::IftypeAp),
                    4 => Some(Nl80211Iftype::IftypeApVlan),
                    6 => Some(Nl80211Iftype::IftypeMonitor),
                    7 => Some(Nl80211Iftype::IftypeMeshPoint),
                    // Add other cases as needed
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect()
}

pub fn wrap_in_box(input: &str) -> String {
    // Split the input string into lines
    let lines: Vec<&str> = input.split('\n').collect();

    // Determine the length of the longest line
    let max_length = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    // Calculate the total width of the box
    let box_width = max_length + 4; // 4 extra characters for borders and spaces

    // Create a new string with a top border
    let mut boxed_string = format!("┏{}┓\n", "━".repeat(box_width - 2));

    // Add each line, padded with spaces to fit the box
    for line in lines {
        let padding_length = box_width - line.chars().count() - 4; // 4 extra characters for borders and spaces
        let padding = " ".repeat(padding_length);
        boxed_string.push_str(&format!("┃ {}{} ┃\n", line, padding));
    }

    // Add a bottom border
    boxed_string.push_str(&format!("┗{}┛", "━".repeat(box_width - 2)));

    boxed_string
}
