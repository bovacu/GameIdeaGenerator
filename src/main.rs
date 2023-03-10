mod common;
use common::TwitterConfig;

use serde::{Deserialize, Serialize};

use rand::Rng;
use tokio::time;

use std::fs::File;
use std::io::Write;
use std::io::{BufReader, Read};
use std::collections::HashMap;

use egg_mode::media::{media_types, upload_media};
use egg_mode::tweet::DraftTweet;

#[derive(Serialize, Deserialize)]
pub struct StableDiffusionResponse {
    pub output: Vec<String>
}


#[derive(Serialize, Deserialize)]
pub struct GeneratorInfo {
    pub genres:                 Vec<String>,
    pub themes:                 Vec<String>,
    pub graphics_dimensions:    Vec<String>,
    pub graphics_styles:        Vec<String>,
    pub key_elements:           Vec<String>,
    pub key_element_sentences:  Vec<String>,
    pub mechanics:              Vec<String>,
    pub platforms:              Vec<String>,
    pub roles:                  Vec<String>,
    pub player_goals:           Vec<String>,
    pub game_goals:             Vec<String>,
    pub game_places:            Vec<String>,
    pub template_texts:         Vec<String>,
    pub challenges:             Vec<String>,
    pub synonyms:               HashMap<String, Vec<String>>
}

impl GeneratorInfo {
    pub fn new() -> GeneratorInfo {
        GeneratorInfo {
            genres:                 Vec::new(),
            themes:                 Vec::new(),
            graphics_styles:        Vec::new(),
            graphics_dimensions:    Vec::new(),
            key_elements:           Vec::new(),
            key_element_sentences:  Vec::new(),
            mechanics:              Vec::new(),
            platforms:              Vec::new(),
            roles:                  Vec::new(),
            player_goals:           Vec::new(),
            game_goals:             Vec::new(),
            game_places:            Vec::new(),
            template_texts:         Vec::new(),
            challenges:             Vec::new(),
            synonyms:               HashMap::new(),
        }
    }
}

fn generate_random_idea(generator_info: &GeneratorInfo, rng: &mut impl Rng) -> String {
    let genre                   = &generator_info.genres[rng.gen_range(0, generator_info.genres.len())];
    let theme                   = &generator_info.themes[rng.gen_range(0, generator_info.themes.len())];
    let graphics_dimension      = &generator_info.graphics_dimensions[rng.gen_range(0, generator_info.graphics_dimensions.len())];
    let graphics_style          = &generator_info.graphics_styles[rng.gen_range(0, generator_info.graphics_styles.len())];
    let key_element             = &generator_info.key_elements[rng.gen_range(0, generator_info.key_elements.len())];
    let key_element_sentence    = &generator_info.key_element_sentences[rng.gen_range(0, generator_info.key_element_sentences.len())];
    let mechanic_0              = &generator_info.mechanics[rng.gen_range(0, generator_info.mechanics.len())];
    let mut mechanic_1          = &generator_info.mechanics[rng.gen_range(0, generator_info.mechanics.len())];

    while mechanic_0 == mechanic_1 {
        mechanic_1 = &generator_info.mechanics[rng.gen_range(0, generator_info.mechanics.len())];
    }

    let platform                = &generator_info.platforms[rng.gen_range(0, generator_info.platforms.len())];
    let role                    = &generator_info.roles[rng.gen_range(0, generator_info.roles.len())];
    let player_goal             = &generator_info.player_goals[rng.gen_range(0, generator_info.player_goals.len())];
    let game_goal               = &generator_info.game_goals[rng.gen_range(0, generator_info.game_goals.len())];
    let game_place              = &generator_info.game_places[rng.gen_range(0, generator_info.game_places.len())];
    let template_text           = &generator_info.template_texts[rng.gen_range(0, generator_info.template_texts.len())];
    let synonym_mix             = &generator_info.synonyms["mix"][rng.gen_range(0, generator_info.synonyms["mix"].len())];
    let synonym_happening       = &generator_info.synonyms["happening"][rng.gen_range(0, generator_info.synonyms["happening"].len())];
    let synonym_goal            = &generator_info.synonyms["goal"][rng.gen_range(0, generator_info.synonyms["goal"].len())];



    let mut tweet = template_text.replace("{genre}",                 genre)
                        .replace("{theme}",                 theme)
                        .replace("{graphics_dimension}",    graphics_dimension)
                        .replace("{graphics_style}",        graphics_style)
                        .replace("{key_element}",           key_element)
                        .replace("{key_element_sentence}",  key_element_sentence)
                        .replace("{mechanic}",              mechanic_0)
                        .replace("{mechanic_0}",            mechanic_0)
                        .replace("{mechanic_1}",            mechanic_1)
                        .replace("{platform}",              platform)
                        .replace("{role}",                  role)
                        .replace("{player_goal}",           player_goal)
                        .replace("{game_goal}",             game_goal)
                        .replace("{game_place}",            game_place)
                        .replace("{synonym_mix}",           synonym_mix)
                        .replace("{synonym_happening}",     synonym_happening)
                        .replace("{synonym_goal}",          synonym_goal);

    if rng.gen_range(0, 10) < 2 {
        tweet.push_str("\n\n");
        tweet.push_str("Challenge: ");
        let selected_challenge = rng.gen_range(0, generator_info.challenges.len());
        tweet.push_str(&generator_info.challenges[selected_challenge]);
    }

    return tweet;
}


async fn tweet(idea: &String, config: &TwitterConfig, w: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    let mut tweet = DraftTweet::new(idea.clone());
    let params = [("key", include_str!("stable_diffusion_key")),
                  ("width", "512"),
                  ("height", "512"),
                  ("samples", "3"),
                  ("prompt", idea.as_str())];

    let client = reqwest::Client::new();
    match client.post("https://stablediffusionapi.com/api/v3/text2img")
        .form(&params)
        .timeout(std::time::Duration::from_secs(180))
        .send()
        .await {
            Ok(resp) => {
                writeln!(w, "Image data retreived")?;
                let text = resp.text().await;

                match text {
                    Ok(text_ok) => {

                        if text_ok.contains("output") {
                            let links: StableDiffusionResponse = serde_json::from_str(&text_ok).unwrap();
                            let typ = media_types::image_png();

                            for link in links.output {
                                let bytes = client.get(&link).send().await?.bytes().await.unwrap();
                                match upload_media(&bytes, &typ, &config.token).await {
                                    Ok(handle) => { tweet.add_media(handle.id.clone()); writeln!(w, "Uploaded data")?;},
                                    Err(handle_err) => writeln!(w, "There was an error uploading media {}", handle_err)?,
                                }
                            }
                        } else {
                            writeln!(w, "Error on request for images {}", text_ok)?;
                        }

                        tweet.send(&config.token).await?;
                    },
                    Err(err) => { writeln!(w, "There was an error deserializing the text from the request {}", err)?; tweet.send(&config.token).await?;},
                }
            },
            Err(err) => {
                writeln!(w, "There was an error retrieving the image data {}", err)?;
            },
        }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logs_name = format!("game_idea_generator_output_{}.log", chrono::offset::Utc::now().format("%d-%m-%Y_%Hh-%Mm-%Ss").to_string());
    let mut w = File::create(format!("./logs/{}", logs_name)).unwrap();

    let config = common::TwitterConfig::load().await;

    let mut data = String::new();
    let f = File::open("data.json").expect("Unable to open file");
    let mut br = BufReader::new(f);
    br.read_to_string(&mut data).expect("Unable to read string");

    let generator_info: GeneratorInfo = serde_json::from_str(&data).unwrap();
    let mut rng = rand::thread_rng();

    let mut interval = time::interval(time::Duration::from_secs(60 * 60 * 24));

    #[allow(while_true)]
    while true {
        interval.tick().await;
        writeln!(&mut w, "--------------------- {:?} ---------------------", chrono::offset::Utc::now())?;

        let idea = generate_random_idea(&generator_info, &mut rng);
        writeln!(&mut w, "Idea generated: {}", idea)?;
        
        match tweet(&idea, &config, &mut w).await {
            Ok(()) => writeln!(&mut w, "Tweet emitted correclty: {}", idea)?,
            Err(err) => writeln!(&mut w, "There was an error on the tweet process! {}", err.to_string())?,
        }
        
        writeln!(&mut w, "-----------------------------------------------------------\n\n")?;
    }

    writeln!(&mut w, "THE PROGRAM ENDED! THIS IS A PROBLEM!")?;

    Ok(())
}
