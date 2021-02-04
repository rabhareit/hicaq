use std::env;
use std::process;
use whoami;
use serde::{Deserialize, Serialize};
use reqwest::{Client, StatusCode};

#[macro_use]
extern crate clap;
use clap::Arg;

#[derive(Serialize, Deserialize, Debug)]
struct AttachmentField {
    title: String,
    value: String,
    short: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
struct Attachment {
    color: String,
    title: String,
    mrkdwn_in: Vec<String>,
    fields: Vec<AttachmentField>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
struct WebhookRequest {
    username: String,
    text: String,
    attachments: Vec<Attachment>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = app_from_crate!()
        .arg(Arg::from_usage("<COMMAND> -c --command <COMMAND> 'Previous command name'"))
        .arg(Arg::from_usage("<STATUS> -s --status <STATUS> 'Success(0) or fail(else)'"))
        .arg(Arg::from_usage("<EXECUTED> -a --executed-at <EXECUTED> 'Executed at'"))
        .arg(Arg::from_usage("<ELAPSED> -e --elapsed <ELAPSED> 'Elapsed time'"))
        .arg(Arg::with_name("message").help("Message send with status"));

    let matches = cmd.get_matches();

    let prev_cmd = match matches.value_of("COMMAND") {
        Some(val) => val.to_string(),
        None => {
            println!("Missing option `command`");
            process::exit(1);
        }
    };

    let notif_status = match matches.value_of("STATUS") {
        Some(val) => val.to_string(),
        None => {
            println!("Missing option `status`");
            process::exit(1);
        }
    };

    let exec_at = match matches.value_of("EXECUTED") {
        Some(val) => val.to_string(),
        None => {
            println!("Missing option `executed`");
            process::exit(1);
        }
    };

    let elapsed = match matches.value_of("ELAPSED") {
        Some(val) => {
            let seconds = val.parse::<i32>().unwrap();
            match seconds > 60 {
                true => {
                    let minutes = seconds / 60;
                    match minutes > 60 {
                        true => format!("{}h {}min {}sec", minutes/60, minutes%60, seconds%60),
                        false => format!("{}min {}sec", minutes, seconds%60)
                    }
                },
                false => format!("{}sec", seconds)
            }
       },
        None => {
            println!("Missing option `elapsed`");
            process::exit(1);
        }
    };

    let cstm_txt = match matches.value_of("message") {
        Some(val) => val.to_string(),
        None => "".to_string()
    };

    let notif_title = if notif_status == "0" {"Command succeeded :ok_woman:"} else {"Command failed :no_good:"};
    let notif_color = if notif_status == "0" {"good"} else {"danger"};

    let current_dir = match env::current_dir() {
        Ok(val) => val,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    };

    let webhook_key = "CMD_NOTIFY_SLACK_WEBHOOK_URL";
    let username_key = "CMD_NOTIFY_SLACK_USER_NAME";

    let webhook_url = match env::var(webhook_key) {
        Ok(val) => val,
        Err(_) => {
            println!("Cannot read Slack webhook URL");
            process::exit(1);
        },
    };

    let chairman = match env::var(username_key) {
        Ok(val) => format!("<@{}>", val),
        Err(_) => {
            println!("Connot read Slack notify username. Send without @ mention");
            "".to_string()
        },
    };
    
    let command_field = AttachmentField {title: "Command".to_string(), value: format!("`{}`", prev_cmd), short: false};
    let directory_filed = AttachmentField {title: "Directory".to_string(), value: format!("`{}`", current_dir.display()), short: false};
    let hostname_field = AttachmentField {title: "Hostname".to_string(), value: whoami::hostname(), short: true};
    let user_field = AttachmentField {title: "User".to_string(), value: whoami::username(), short: true};
    let exec_field = AttachmentField {title: "executed at".to_string(), value: exec_at, short: true};
    let elapsed_field = AttachmentField {title: "Elapsed time".to_string(), value: elapsed, short: true};

    let attachment = Attachment {
        color: notif_color.to_string(), 
        title: notif_title.to_string(), 
        mrkdwn_in: vec!("fields".to_string()), 
        fields: vec!(
            command_field, 
            directory_filed, 
            hostname_field, 
            user_field, 
            exec_field, 
            elapsed_field
        )
    };

    let webhook_req = WebhookRequest {
        username: "hicaq".to_string(),
        text: format!("{} \n {}", chairman, cstm_txt),
        attachments: vec!(attachment)
    };

    let client = Client::new();
    let res = client.post(&webhook_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&webhook_req)
        .send()
        .await?;

    match res.status() {
        StatusCode::OK => {
            println!("Success de gozaru!");
            Ok(())
        }
        s => {
            println!("Recieved response status: {:?}, katajikenai...", s);
            process::exit(1); 
        }
    }
}
