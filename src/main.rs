fn main() {
}

#[cfg(test)]
mod tests {
    use std::{fs::{self, File}, path::PathBuf};
    use rust_glyph_recog::glyph_recognizer::GlyphRecognizer;

    #[test]
    fn test() {
        let input_dir = "dats/";
        let recog = GlyphRecognizer::new_from_data_dir(input_dir);

        let dirs:Vec<PathBuf> = fs::read_dir(input_dir).unwrap()
            .filter(|x| x.as_ref().unwrap().path().is_dir())
            .map(|x| x.unwrap().path())
            .collect();


        for dir in dirs.into_iter().filter(|dir| dir.file_name().unwrap().to_str().unwrap() != "overlaps") {
            // //for dir in dirs {
            let dir_name = dir.file_name().unwrap().to_str().unwrap().to_owned();
            let c = std::char::from_u32(dir_name.parse::<u32>().unwrap()).unwrap().to_string();
            for file in fs::read_dir(input_dir.to_owned() + dir.file_name().unwrap().to_str().unwrap()).unwrap().into_iter().map(|x| x.unwrap()) {
                let mut file = File::open(format!("{}{}/{}", input_dir, dir_name, file.path().file_name().unwrap().to_str().unwrap())).unwrap();
                let result = recog.parse_glyph_from_stream(&mut file);
                assert_eq!(result, c);
                //println!("Expected {} got {}", c, result);
            }
        }
    }
}
