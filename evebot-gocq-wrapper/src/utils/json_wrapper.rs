#[macro_export]
macro_rules! build_single_text {
    ($text: expr) => {
        serde_json::json! {[{
            "type": "text",
            "data": {
                "text": $text
            }
        }]}
    };
}

#[macro_export]
macro_rules! build_single_image {
    ($image: expr) => {
        serde_json::json! {[{
            "type": "image",
            "data": {
                "file": $image
            }
        }]}
    };
}
