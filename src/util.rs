use super::Nl80211Iftype;

pub fn decode_iftypes(bytes: Vec<u8>) -> Vec<Nl80211Iftype> {
    bytes
        .chunks(4)
        .filter_map(|chunk| {
            if chunk.len() == 4 {
                // Convert the chunk into an array of length 4
                let chunk_array: [u8; 4] = chunk.try_into().expect("slice with incorrect length");

                // Interpret the chunk as two u16s
                let (len, typecode) = (
                    u16::from_ne_bytes([chunk_array[0], chunk_array[1]]),
                    u16::from_ne_bytes([chunk_array[2], chunk_array[3]]),
                );

                match typecode {
                    0 => Some(Nl80211Iftype::IftypeUnspecified),
                    1 => Some(Nl80211Iftype::IftypeAdhoc),
                    2 => Some(Nl80211Iftype::IftypeStation),
                    3 => Some(Nl80211Iftype::IftypeAp),
                    4 => Some(Nl80211Iftype::IftypeApVlan),
                    6 => Some(Nl80211Iftype::IftypeMonitor),
                    7 => Some(Nl80211Iftype::IftypeMeshPoint),
                    8 => Some(Nl80211Iftype::IftypeP2pClient),
                    9 => Some(Nl80211Iftype::IftypeP2pGo),
                    10 => Some(Nl80211Iftype::IftypeP2pDevice),
                    11 => Some(Nl80211Iftype::IftypeOcb),
                    12 => Some(Nl80211Iftype::IftypeNan),
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
