//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Write;
use std::sync::Arc;

use serenity::async_trait;
use serenity::client::bridge::gateway::{ShardId, ShardManager};
use serenity::framework::standard::buckets::{LimitedFor, RevertBucket};
use serenity::framework::standard::macros::{check, command, group, help, hook};
use serenity::framework::standard::{
    help_commands,
    Args,
    CommandGroup,
    CommandOptions,
    CommandResult,
    DispatchError,
    HelpOptions,
    Reason,
    StandardFramework,
};
use serenity::http::Http;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::component::{ButtonStyle, InputTextStyle};
use serenity::model::channel::{Channel, Message, AttachmentType};
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::id::UserId;
use serenity::model::permissions::Permissions;
use serenity::prelude::*;
use serenity::utils::{content_safe, ContentSafeOptions};
use tokio::sync::Mutex;
use serenity::futures::StreamExt;

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Got command '{}' by user '{}'", command_name, msg.author.name);

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    }
}

// You can construct a hook without the use of a macro, too.
// This requires some boilerplate though and the following additional import.
use serenity::futures::future::BoxFuture;
use serenity::FutureExt;
fn _dispatch_error_no_macro<'fut>(
    ctx: &'fut mut Context,
    msg: &'fut Message,
    error: DispatchError,
    _command_name: &str,
) -> BoxFuture<'fut, ()> {
    async move {
        if let DispatchError::Ratelimited(info) = error {
            if info.is_first_try {
                let _ = msg
                    .channel_id
                    .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                    .await;
            }
        };
    }
    .boxed()
}

#[group]
#[commands(load, update, rescue, delete_savefiles_i_know_what_i_do, unknown, fetch_maps, find_english)]
struct General;


use celeste_save_data_rs::save_data::SaveData;
use celeste_save_data_rs::map_data::GameData;
use celeste_visualizer::generate_png;
use celeste_savefile_db::*;
use celeste_visualizer::diff::generate_diff_png;

struct GameDataStore;

impl TypeMapKey for GameDataStore {
    type Value = Arc<RwLock<GameData>>;
}

struct CelesteDBStore;

impl TypeMapKey for CelesteDBStore {
    type Value = Arc<RwLock<CelesteSavefileDB>>;
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c
                   .with_whitespace(true)
                   .on_mention(Some(bot_id))
                   .prefix("~")
                   // In this case, if "," would be first, a message would never
                   // be delimited at ", ", forcing you to trim your arguments if you
                   // want to avoid whitespaces at the start of each.
                   .delimiters(vec![", ", ","])
                   // Sets the bot's owners. These will be used for commands that
                   // are owners only.
                   .owners(owners))

    // Set a function to be called prior to each command execution. This
    // provides the context of the command, the message that was received,
    // and the full name of the command that will be called.
    //
    // Avoid using this to determine whether a specific command should be
    // executed. Instead, prefer using the `#[check]` macro which
    // gives you this functionality.
    //
    // **Note**: Async closures are unstable, you may use them in your
    // application if you are fine using nightly Rust.
    // If not, we need to provide the function identifiers to the
    // hook-functions (before, after, normal, ...).
        .before(before)
    // Similar to `before`, except will be called directly _after_
    // command execution.
        .after(after)
    // Set a function that's called whenever an attempted command-call's
    // command could not be found.
        .unrecognised_command(unknown_command)
    // Set a function that's called whenever a message is not a command.
        .normal_message(normal_message)
    // Set a function that's called whenever a command's execution didn't complete for one
    // reason or another. For example, when a user has exceeded a rate-limit or a command
    // can only be performed by the bot owner.
        .on_dispatch_error(dispatch_error)
    // Can't be used more than once per 5 seconds:
        .bucket("emoji", |b| b.delay(5)).await
    // Can't be used more than 2 times per 30 seconds, with a 5 second delay applying per channel.
    // Optionally `await_ratelimits` will delay until the command can be executed instead of
    // cancelling the command invocation.
        .bucket("complicated", |b| b.limit(2).time_span(30).delay(5)
            // The target each bucket will apply to.
            .limit_for(LimitedFor::Channel)
            // The maximum amount of command invocations that can be delayed per target.
            // Setting this to 0 (default) will never await/delay commands and cancel the invocation.
            .await_ratelimits(1)
            // A function to call when a rate limit leads to a delay.
            .delay_action(delay_action)).await
    // The `#[group]` macro generates `static` instances of the options set for the group.
    // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
    // #name is turned all uppercase
        .group(&GENERAL_GROUP);

    // For this example to run properly, the "Presence Intent" and "Server Members Intent"
    // options need to be enabled.
    // These are needed so the `required_permissions` macro works on the commands that need to
    // use it.
    // You will need to enable these 2 options on the bot application, and possibly wait up to 5
    // minutes.
    let intents = GatewayIntents::all();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        let yml = std::fs::read_to_string("../maps.yaml").unwrap();
        let game_data = GameData::from_str(&yml).unwrap();
        data.insert::<GameDataStore>(Arc::new(RwLock::new(game_data)));

        let db = CelesteSavefileDB::new().await.unwrap();
        data.insert::<CelesteDBStore>(Arc::new(RwLock::new(db)));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

async fn update_maps_yaml(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let url = "https://raw.githubusercontent.com/niuez/celeste_data/main/maps.yaml";
    let body = reqwest::get(url)
        .await?
        .text()
        .await?;
    eprintln!("{}", body);
    let game_data = GameData::from_str(&body).unwrap();
    data.insert::<GameDataStore>(Arc::new(RwLock::new(game_data)));
    msg.channel_id.say(&ctx.http, format!("update complete")).await?;
    Ok(())
}

#[command]
async fn fetch_maps(ctx: &Context, msg: &Message) -> CommandResult {
    update_maps_yaml(ctx, msg).await
        .map_err(|e| format!("fetch failed {:?}", e))?;
    Ok(())
}


async fn check_save_data(msg: &Message) -> Result<HashMap<String, (SaveData, Savefile)>, String> {
    let mut map = HashMap::new();
    for attachment in msg.attachments.iter() {
        match attachment.download().await {
            Err(why) => {
                Err(format!("download error {:?}", why))?
            }
            Ok(data) => {
                let xml = String::from_utf8(data).map_err(|e| format!("from_utf8 error {:?}", e))?;
                let now_data = SaveData::from_str(&xml)?;
                let savefile = Savefile {
                    discord_id: msg.author.id.to_string(),
                    filename: attachment.filename.clone(),
                    xml,
                };
                map.insert(attachment.filename.clone(), (now_data, savefile));
            }
        }
    }
    if map.is_empty() {
        Err("no attachments".to_string())
    }
    else {
        Ok(map)
    }
}

async fn load_data_dialog(ctx: &Context, msg: &Message, save_data: SaveData) -> CommandResult {
    let mut table = Vec::new();
    //table.push(("Chapter", "TotalStrawberries", "Completed", "SingleRunCompleted", "FullClear", "Deaths", "TimePlayed", "BestTime", "BestFullClearTime", "BestDashes", "BestDeaths", "HeartGem"));
    table.push(vec!["Chapter".to_string(), "BestTime".to_string(), "Best/Deaths".to_string(), "Strawberries".to_string()]);
    let sides = vec!["A", "B", "C"];
    let mut m = {
        let levels = {
            let data_read = ctx.data.read().await;
            let game_data_lock = data_read.get::<GameDataStore>()
                .expect("Expect GameDataStore in TypeMap").clone();
            let game_data = game_data_lock.read().await;
            game_data.levels().map(|s| (s.level.to_string(), s.name.to_string())).collect::<Vec<_>>()
        };

        msg.channel_id.send_message(&ctx, move |m| {
            m.content("select").components(|c| {
                c.create_action_row(|row| {
                    row.create_select_menu(|menu| {
                        menu.custom_id("lang_select");
                        menu.placeholder("lang");
                        menu.options(|f| {
                            f.create_option(|o| o.label("en").value("en").default_selection(true));
                            f.create_option(|o| o.label("ja").value("ja"))
                        })
                    })
                });
                for i in 0..4 {
                    if i * 25 < levels.len() {
                        c.create_action_row(|row| {
                            row.create_select_menu(|menu| {
                                menu.custom_id(format!("level_select{}", i));
                                menu.placeholder("level");
                                menu.options(|f| {
                                    for j in 0..25 {
                                        if i * 25 + j < levels.len() {
                                            f.create_option(|o| o.label(levels[i * 25 + j].0.to_string()).value(levels[i * 25 + j].1.to_string()));
                                        }
                                    }
                                    f
                                })
                            })
                        });
                    }
                }
                c
            })
        })
    };
    let m = m.await.unwrap();
    let mut interaction_stream = m.await_component_interactions(&ctx)
        .author_id(msg.author.id)
        .timeout(std::time::Duration::from_secs(30))
        .build();
    {
        let mut selected_lang = "en".to_string();
        while let Some(interaction) = interaction_stream.next().await {
            if interaction.data.custom_id.strip_prefix("level_select").is_some() {
                let selected_level = interaction.data.values[0].to_string();

                let png_file = tempfile::NamedTempFile::new().map_err(|e| format!("cant create tempfile {:?}", e))?;
                {
                    let data_read = ctx.data.read().await;
                    let game_data_lock = data_read.get::<GameDataStore>()
                        .expect("Expect GameDataStore in TypeMap").clone();
                    let game_data = game_data_lock.read().await;

                    generate_png(&save_data, game_data.get_level_data(&selected_level).unwrap().maps(), png_file.path(), &selected_lang)
                        .map_err(|e| format!("cant generate png {:?}", e))?;
                }
                let tokio_file = tokio::fs::File::open(png_file.path()).await
                    .map_err(|e| format!("cant create tokio file {:?}", e))?;
                interaction.create_interaction_response(&ctx, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|d| {
                        d.add_file(AttachmentType::File {
                            file: &tokio_file,
                            filename: format!("{}_{}.png", msg.author, &selected_level),
                        })
                    })
                }).await?;
                break;
            }
            else {
                selected_lang = interaction.data.values[0].to_string();
                interaction.create_interaction_response(&ctx, |r| {
                    r.kind(InteractionResponseType::UpdateMessage).interaction_response_data(|d| {
                        d.content(format!("{}", selected_lang))
                    })
                }).await?;
            }
        }
        m.delete(&ctx).await?;
        return Ok(());
    };
}

#[command]
async fn load(ctx: &Context, msg: &Message) -> CommandResult {
    let discord_id = msg.author.id.to_string();
    let save_data = {
        let data_read = ctx.data.read().await;
        let game_data_lock = data_read.get::<GameDataStore>()
            .expect("Expect GameDataStore in TypeMap").clone();
        let game_data = game_data_lock.read().await;
        let db_lock = data_read.get::<CelesteDBStore>()
            .expect("Expect CelesteDBStore in TypeMap").clone();
        let db = db_lock.read().await;
        let savefiles = db.get_savefiles(&discord_id).await
            .map_err(|e| format!("cant get data from db {:?}", e))?;
        let mut save_data = SaveData::new();
        for sf in savefiles {
            save_data.merge(SaveData::from_str(&sf.xml)?);
        }
        save_data
    };
    load_data_dialog(ctx, msg, save_data).await?;
    Ok(())
}

#[command]
async fn update(ctx: &Context, msg: &Message) -> CommandResult {
    let discord_id = msg.author.id.to_string();
    match check_save_data(msg).await {
        Err(why) => {
            msg.channel_id.say(&ctx.http, why).await?;
        }
        Ok(new_savefiles) => {
            let png_diff_file = tempfile::NamedTempFile::new().map_err(|e| format!("cant create tempfile {:?}", e))?;
            {
                let data_read = ctx.data.read().await;
                let game_data_lock = data_read.get::<GameDataStore>()
                    .expect("Expect GameDataStore in TypeMap").clone();
                let game_data = game_data_lock.read().await;
                let db_lock = data_read.get::<CelesteDBStore>()
                    .expect("Expect CelesteDBStore in TypeMap").clone();
                let db = db_lock.read().await;
                let now_savefiles = db.get_savefiles(&discord_id).await
                    .map_err(|e| format!("cant get data from db {:?}", e))?;

                let mut before = SaveData::new();
                let mut after = SaveData::new();
                for sf in now_savefiles {
                    let data = SaveData::from_str(&sf.xml)?;
                    if !new_savefiles.contains_key(&sf.filename) {
                        after.merge(data.clone());
                    }
                    before.merge(data);
                }
                for e in new_savefiles.values() {
                    after.merge(e.0.clone())
                }
                generate_diff_png(&game_data, &before, &after, png_diff_file.path(), "en")?;
            };
            let tokio_diff_file = tokio::fs::File::open(png_diff_file.path()).await
                .map_err(|e| format!("cant create tokio file {:?}", e))?;
            let m = {
                msg.channel_id.send_message(&ctx, |m| {
                    m.add_file(AttachmentType::File {
                        file: &tokio_diff_file,
                        filename: format!("{}_diff.png", msg.author),
                    });
                    m.content("select").components(|c| {
                        c.create_action_row(|row| {
                            row.create_button(|b| {
                                b.custom_id("apply");
                                b.label("apply");
                                b.style(ButtonStyle::Primary)
                            });
                            row.create_button(|b| {
                                b.custom_id("dismiss");
                                b.label("dismiss");
                                b.style(ButtonStyle::Secondary)
                            })
                        })
                    })
                })
            };
            let m = m.await.unwrap();
            let interaction = match m.await_component_interaction(&ctx)
                .author_id(msg.author.id)
                .timeout(std::time::Duration::from_secs(30))
                .await {
                    Some(x) => x,
                    None => {
                        m.reply(&ctx, "Timed out").await.unwrap();
                        return Ok(());
                }
            };
            let selected_button = interaction.data.custom_id.clone();

            if selected_button == "apply" {
                {
                    let data_read = ctx.data.read().await;
                    let game_data_lock = data_read.get::<GameDataStore>()
                        .expect("Expect GameDataStore in TypeMap").clone();
                    let game_data = game_data_lock.read().await;
                    let db_lock = data_read.get::<CelesteDBStore>()
                        .expect("Expect CelesteDBStore in TypeMap").clone();
                    let db = db_lock.read().await;
                    for (filename, (_, sf)) in new_savefiles.into_iter() {
                        db.update_savefile(sf).await
                            .map_err(|e| format!("cant save {:?}", e))?;
                    }
                }
                m.reply(&ctx, "applied").await?;
            }
            else {
                m.reply(&ctx, "dismissed").await?;
            }
        }
    }
    Ok(())
}
use tokio::io::{self, AsyncWriteExt};

#[command]
async fn rescue(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let discord_id = msg.author.id.to_string();
    let mut ans_files = vec![];
    {
        let data_read = ctx.data.read().await;
        let game_data_lock = data_read.get::<GameDataStore>()
            .expect("Expect GameDataStore in TypeMap").clone();
        let game_data = game_data_lock.read().await;
        let db_lock = data_read.get::<CelesteDBStore>()
            .expect("Expect CelesteDBStore in TypeMap").clone();
        let db = db_lock.read().await;
        let now_savefiles = db.get_savefiles(&discord_id).await
            .map_err(|e| format!("cant get data from db {:?}", e))?;

        for sf in now_savefiles {
            let ans_file = tempfile::NamedTempFile::new().map_err(|e| format!("cant create tempfile {:?}", e))?;
            let mut tokio_file = tokio::fs::File::create(ans_file.path()).await
                .map_err(|e| format!("cant create tokio file {:?}", e))?;
            tokio_file.write_all(sf.xml.as_bytes()).await?;
            ans_files.push((sf.filename, ans_file));
        }
    }
    {
        let mut files = vec![];
        for (name, file) in ans_files {
            files.push((name, tokio::fs::File::open(file.path()).await?));
        }
        msg.channel_id.send_message(&ctx, |m| {
            for (name, file) in files.iter() {
                m.add_file(AttachmentType::File {
                    file: &file,
                    filename: name.clone(),
                });
            }
            m
        }).await?;
    }
    Ok(())
}

#[command]
async fn delete_savefiles_i_know_what_i_do(ctx: &Context, msg: &Message) -> CommandResult {
    let discord_id = msg.author.id.to_string();
    {
        let data_read = ctx.data.read().await;
        let game_data_lock = data_read.get::<GameDataStore>()
            .expect("Expect GameDataStore in TypeMap").clone();
        let game_data = game_data_lock.read().await;
        let db_lock = data_read.get::<CelesteDBStore>()
            .expect("Expect CelesteDBStore in TypeMap").clone();
        let db = db_lock.read().await;
        db.delete_savefiles(&discord_id).await
            .map_err(|e| format!("cant delete data from db {:?}", e))?;
    }
    Ok(())
}

#[command]
async fn unknown(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let discord_id = msg.author.id.to_string();
    {
        let data_read = ctx.data.read().await;
        let game_data_lock = data_read.get::<GameDataStore>()
            .expect("Expect GameDataStore in TypeMap").clone();
        let game_data = game_data_lock.read().await;
        let db_lock = data_read.get::<CelesteDBStore>()
            .expect("Expect CelesteDBStore in TypeMap").clone();
        let db = db_lock.read().await;
        let now_savefiles = db.get_savefiles(&discord_id).await
            .map_err(|e| format!("cant get data from db {:?}", e))?;
        let mut savedata = SaveData::new();
        for sf in now_savefiles {
            savedata.merge(SaveData::from_str(&sf.xml)?);
        }
        let savedata = savedata;
        let known_levels: HashSet<_> = game_data.levels().map(|s| s.level.clone()).collect();
        let mut unknown_levels = HashSet::new();
        for level in savedata.levels.keys() {
            if !known_levels.contains(level) {
                unknown_levels.insert(level.clone());
            }
        }
        if let Ok(level) = args.single::<String>() {
            if !unknown_levels.contains(&level) {
                msg.channel_id.say(&ctx.http, "not found").await?;
            }
            else {
                let ans_file = tempfile::NamedTempFile::new().map_err(|e| format!("cant create tempfile {:?}", e))?;
                {
                    let mut tokio_file = tokio::fs::File::create(ans_file.path()).await
                        .map_err(|e| format!("cant create tokio file {:?}", e))?;
                    tokio_file.write(format!("- level: {}\n  name: ''\n", level).as_bytes()).await?;
                    for map_code in savedata.levels[&level].iter() {
                        if map_code.side == 0 {
                            tokio_file.write(format!("    - sid: '{}'\n", map_code.sid).as_bytes()).await?;
                            tokio_file.write(format!("      name:\n")                  .as_bytes()).await?;
                            tokio_file.write(format!("        en: ''\n")               .as_bytes()).await?;
                            tokio_file.write(format!("      sides: [0]\n")             .as_bytes()).await?;
                        }
                    }
                }
                {
                    let tokio_file = tokio::fs::File::open(ans_file.path()).await?;
                    msg.channel_id.send_message(&ctx, |m| {
                        m.add_file(AttachmentType::File {
                            file: &tokio_file,
                            filename: format!("{}.yaml", level),
                        })
                    }).await?;
                }
            }
        }
        else {
            let mut levels = unknown_levels.into_iter().collect::<Vec<_>>();
            levels.sort();
            let message = levels.join("\n");
            msg.channel_id.say(&ctx.http, message).await?;
        }
    }
    Ok(())
}


#[command]
async fn find_english(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let discord_id = msg.author.id.to_string();
    {
        let data_read = ctx.data.read().await;
        let game_data_lock = data_read.get::<GameDataStore>()
            .expect("Expect GameDataStore in TypeMap").clone();
        let game_data = game_data_lock.read().await;
        let db_lock = data_read.get::<CelesteDBStore>()
            .expect("Expect CelesteDBStore in TypeMap").clone();
        let db = db_lock.read().await;
        let now_savefiles = db.get_savefiles(&discord_id).await
            .map_err(|e| format!("cant get data from db {:?}", e))?;
        let mut savedata = SaveData::new();
        for sf in now_savefiles {
            savedata.merge(SaveData::from_str(&sf.xml)?);
        }
        let savedata = savedata;
        let known_levels: HashSet<_> = game_data.levels().map(|s| s.level.clone()).collect();
        let mut unknown_levels = HashSet::new();
        for level in savedata.levels.keys() {
            if !known_levels.contains(level) {
                unknown_levels.insert(level.clone());
            }
        }
        let mut english_dist = HashMap::new();
        for attachment in msg.attachments.iter() {
            match attachment.download().await {
                Err(why) => {
                    Err(format!("download error {:?}", why))?
                }
                Ok(data) => {
                    let english = String::from_utf8(data).map_err(|e| format!("from_utf8 error {:?}", e))?;
                    let english = english.replace("=", "= ");
                    let splits = english.lines()
                        .filter(|s| !s.strip_prefix("#").is_some())
                        .map(|s| s.split_whitespace().collect::<Vec<_>>())
                        .flatten()
                        .rev()
                        .collect::<Vec<_>>();
                    let mut now: Vec<String> = vec![];
                    for s in splits {
                        if let Some(key) = s.strip_suffix("=") {
                            now.reverse();
                            english_dist.insert(key.to_string(), now.join(" "));
                            eprintln!("{} {}", key.to_string(), now.join(" "));
                            now.clear();
                        }
                        else {
                            now.push(s.to_string());
                        }
                    }
                }
            }
        }

        let ans_file = tempfile::NamedTempFile::new().map_err(|e| format!("cant create tempfile {:?}", e))?;
        for unknown_level in unknown_levels {
            let mut ok = true;
            for map_code in savedata.levels[&unknown_level].iter() {
                let key = map_code.sid.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>();
                eprintln!("{} -> {}", map_code.sid, key);
                if let Some(value) = english_dist.get(key.as_str()) {
                }
                else {
                    ok = false;
                }
            }
            if ok {
                {
                    let mut tokio_file = tokio::fs::File::create(ans_file.path()).await
                        .map_err(|e| format!("cant create tokio file {:?}", e))?;
                    tokio_file.write(format!("- level: {}\n  name: '{}'\n  maps:\n", unknown_level, unknown_level).as_bytes()).await?;
                    for map_code in savedata.levels[&unknown_level].iter() {
                        let key = map_code.sid.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>();
                        if map_code.side == 0 {
                            tokio_file.write(format!("    - sid: '{}'\n", map_code.sid).as_bytes()).await?;
                            tokio_file.write(format!("      name:\n")                  .as_bytes()).await?;
                            tokio_file.write(format!("        en: '{}'\n", english_dist[key.as_str()])               .as_bytes()).await?;
                            tokio_file.write(format!("      sides: [0]\n")             .as_bytes()).await?;
                        }
                    }
                }
            }
        }

        {
            let tokio_file = tokio::fs::File::open(ans_file.path()).await?;
            msg.channel_id.send_message(&ctx, |m| {
                m.add_file(AttachmentType::File {
                    file: &tokio_file,
                    filename: format!("maps.yaml"),
                })
            }).await?;
        }


    }
    Ok(())
}
