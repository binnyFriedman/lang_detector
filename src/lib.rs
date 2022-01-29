use std::collections::HashMap;
use std::fs;

const MAX_DISTANCE: usize = 301;

fn generate_profile(document: String) -> Vec<String> {
    const MAX_LENGTH: i32 = 100000;
    const N_GRAM_SIZE: usize = 3;
    const MAX_PROFILE_LEN: usize = 300;

    let mut profile = HashMap::new();

    let mut word_count: i32 = 0;

    'word_itr: for word in document.split_whitespace() {
        if word_count == MAX_LENGTH {
            break 'word_itr;
        }
        let mut index: usize = 0;
        let word_length = word.chars().count();

        while index < word_length + 1 {
            let mut word_slice = String::new();
            if index == 0 {
                word_slice.push(' ');
            }
            let mut from_index: i32 = index.clone() as i32 - 1;
            if from_index < 0 {
                from_index = 0;
            }
            let mut end_index: usize = from_index as usize + N_GRAM_SIZE;
            if end_index > word_length {
                end_index = word_length;
            }
            if index == 0 {
                end_index -= 1;
            }

            while from_index < end_index as i32 {
                let c = word.chars().nth(from_index as usize).unwrap();
                if !c.is_alphabetic() && from_index != word_length as i32 - 1 {
                    continue 'word_itr;
                }
                if c.is_alphabetic() {
                    profile
                        .entry(c.to_string())
                        .and_modify(|x| *x += 1)
                        .or_insert(1);
                    word_slice.push(c);
                }
                from_index += 1;
            }

            if end_index == word_length {
                while word_slice.chars().count() < N_GRAM_SIZE {
                    word_slice.push(' ');
                }
            }
            profile
                .entry(word_slice.clone())
                .and_modify(|count| *count += 1)
                .or_insert(1);
            index += 1;
        }
        word_count += 1;
    }

    //turn the hashmap into a vector of tuples
    let mut profile_vec = profile.into_iter().collect::<Vec<(String, i32)>>();

    //sort the vector by the second element in the tuple
    profile_vec.sort_by(|a, b| b.1.cmp(&a.1));

    //remove the frequency count from the vector.
    profile_vec
        .into_iter()
        .take(MAX_PROFILE_LEN)
        .map(|tuple| tuple.0.clone())
        .collect()
}

pub fn detect(doc: String) -> String {
    let profile = generate_profile(doc);
    let profile_length = profile.len();
    let profiles = get_profiles_from_file();
    let profile_distances = get_min_distances(profile, profiles);

    let mut min_profile = (String::new(), i32::MAX);
    for (profile, distance) in profile_distances {
        if distance < min_profile.1 {
            min_profile = (profile, distance);
        }
    }

    //most out of place is no match in any profile
    let most_out_of_place = MAX_DISTANCE * profile_length;

    //lets say that we need to be within 10% of the most out of place
    if min_profile.1 > (most_out_of_place - most_out_of_place / 10) as i32 {
        return String::from("No match");
    }

    min_profile.0
}

fn get_min_distances(
    profile: Vec<String>,
    profiles: HashMap<String, Vec<String>>,
) -> HashMap<String, i32> {
    let mut profile_distances = HashMap::new();
    for (lang, lang_profile) in profiles {
        let mut distance: i32 = 0;
        let mut index: usize = 0;
        for word in &profile {
            match lang_profile.iter().position(|x| x == word) {
                Some(x) => distance += (x as i32 - index as i32).abs(),
                None => distance += MAX_DISTANCE as i32,
            }
            index += 1;
        }
        profile_distances.insert(lang, distance);
    }
    profile_distances
}

fn get_profiles_from_file() -> HashMap<String, Vec<String>> {
    let data = fs::read_to_string("data/lang_profiles.json").expect("Unable to read file");
    let profiles: HashMap<String, Vec<String>> =
        serde_json::from_str(&data).expect("Unable to parse json");
    profiles
}

fn save_profiles_to_file(profiles: HashMap<String, Vec<String>>) {
    let data = serde_json::to_string(&profiles).expect("Unable to serialize json");

    fs::write("data/lang_profiles.json", data).expect("Unable to write to file");
}

fn add_profile(lange: String, profile: Vec<String>) {
    let mut profiles = get_profiles_from_file();
    profiles.insert(lange, profile);
    save_profiles_to_file(profiles);
}

fn add_profile_from_path(tag: String) {
    let raw_profile =
        fs::read_to_string(format!("data/raw_languages/{}.txt", tag)).expect("Unable to read file");
    let profile = generate_profile(raw_profile);
    add_profile(tag, profile);
}

fn generate_all_languages() {
    for entry in fs::read_dir("data/raw_languages").expect("Unable to read directory") {
        let entry = entry.expect("Unable to read entry");
        let tag = entry
            .file_name()
            .into_string()
            .expect("Unable to read file name");
        let tag = tag.split(".").collect::<Vec<&str>>()[0];
        add_profile_from_path(tag.to_string());
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn generate_all_profiles() {
        super::generate_all_languages();
        let profiles = super::get_profiles_from_file();
        assert_eq!(profiles.len(), 7);
    }

    #[test]
    fn generate_profile() {
        let noised_doc = "123dog";
        let profile = super::generate_profile(noised_doc.to_string());
        assert_eq!(profile.len(), 0);

        let hebrew_doc = "שלום עולם";
        let profile = super::generate_profile(hebrew_doc.to_string());
        assert_eq!(profile.len(), hebrew_doc.chars().count());
    }

    #[test]
    fn allow_ending_punctuation() {
        let doc = "my dog.";
        let profile = super::generate_profile(doc.to_string());
        print!("{:?}", profile);
        assert_eq!(profile.len(), doc.chars().count() + 1);
    }

    #[test]
    fn add_english_profile() {
        let english_profile =
            fs::read_to_string("data/raw_languages/english.txt").expect("Unable to read file");
        let english_profile = super::generate_profile(english_profile);
        super::add_profile("english".to_string(), english_profile);
        let profiles = super::get_profiles_from_file();
        println!("{:?}", profiles.keys());
        assert_eq!(profiles.len(), 1);
    }

    #[test]
    fn add_dutch_profile() {
        let dutch_profile =
            fs::read_to_string("data/raw_languages/dutch.txt").expect("Unable to read file");
        let dutch_profile = super::generate_profile(dutch_profile);
        let profile_ref = dutch_profile.clone();
        super::add_profile("dutch".to_string(), dutch_profile);
        let profiles = super::get_profiles_from_file();
        assert_eq!(profiles.get("dutch"), Some(&profile_ref));
    }

    #[test]
    fn add_spanish_profile() {
        let spanish =
            fs::read_to_string("data/raw_languages/spanish.txt").expect("Unable to read file");
        let spanish = super::generate_profile(spanish);
        let profile_ref = spanish.clone();
        super::add_profile("spanish".to_string(), spanish);
        let profiles = super::get_profiles_from_file();
        assert_eq!(profiles.get("spanish"), Some(&profile_ref));
    }

    #[test]
    fn detect_english() {
        let doc = "Searches for an element in an iterator, returning its index.

position() takes a closure that returns true or false. It applies this closure to each element of the iterator, and if one of them returns true, then position() returns Some(index). If all of them return false, it returns None.

position() is short-circuiting; in other words, it will stop processing as soon as it finds a true.
Overflow Behavior

The method does no guarding against overflows, so if there are more than usize::MAX non-matching elements, it either produces the wrong result or panics. If debug assertions are enabled, a panic is guaranteed.
Panics

This function might panic if the iterator has more than usize::MAX non-matching elements.
Examples

Basic usage:";
        let result = super::detect(doc.to_string());
        assert_eq!(result, "english");
    }

    #[test]
    fn detect_dutch() {
        let doc = "Dit is een test";
        let result = super::detect(doc.to_string());
        assert_eq!(result, "dutch");
    }

    #[test]
    fn detect_spanish() {
        let doc = "Hola mundo";
        let result = super::detect(doc.to_string());
        assert_eq!(result, "spanish");
    }

    #[test]
    fn detect_french() {
        super::add_profile_from_path("french".to_string());
        let doc = "Bonjour le monde";
        let result = super::detect(doc.to_string());
        assert_eq!(result, "french");
    }

    #[test]
    fn detect_hebrew() {
        super::add_profile_from_path("hebrew".to_string());
        let doc = "שלום עולם";
        let result = super::detect(doc.to_string());
        assert_eq!(result, "hebrew");
    }
    #[test]
    fn detect_japanese() {
        super::add_profile_from_path("japanese".to_string());
        let doc = "　だとすれば、野党も我々メディアも、これまでの権力監視の物差しを変える必要があるだろう。手法が強権か否か。敵視の程度はいかばかりか。そうした、安倍、菅義偉の両政権の9年間ですっかり習い性になった「分かりやすい」物差しだけで瞬間反応的に評価しようとすると、「分かりにくい」岸田首相の術中にはまるだけではあるまいか。";
        let result = super::detect(doc.to_string());
        assert_eq!(result, "japanese");
    }

    #[test]
    fn detect_chinese() {
        super::add_profile_from_path("chinese".to_string());
        let doc = "咖啡是世界上最重要的作物之一，不仅因为受到世界上数以亿计的消费者的追捧成为很多人日常必喝的主要饮料，更为重要的是咖啡种植是数百万小规模经营农民们赖以生存的的生计。
另一方面，由于较富裕国家的消费者们口味和认知的变化，近几十年来富裕国家对牛油果和腰果的消费需求也大幅增加。";
        assert_eq!(super::detect(doc.to_string()), "chinese");
    }

    #[test]
    fn detect_no_language() {
        let doc = "4325235 23423!!$! 54* 53252%$$@#";
        assert_eq!(super::detect(doc.to_string()), "No match")
    }
}
