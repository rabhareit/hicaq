use std::env;
use std::process;
use whoami;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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


    let args: Vec<String> = env::args().collect();
    let prev_command = &args[1].to_string();
    let notif_status = &args[2].to_string();
    let notif_title = if notif_status == "0" {"Command succeeded :ok_woman:"} else {"Command failed :no_good:"};
    let notif_color = if notif_status == "0" {"good"} else {"danger"};

    let exec_at = &args[3].to_string();
    let elapsed = &args[4].to_string();

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
        Err(err) => {
            println!("{}: {}", err, webhook_key);
            process::exit(1);
        },
    };

    let chairman = match env::var(username_key) {
        Ok(val) => val,
        Err(err) => {
            println!("{}: {}", err, username_key);
            process::exit(1);
        },
    };
    
    let command_field = AttachmentField {title: "Command".to_string(), value: prev_command.to_string(), short: false};
    let directory_filed = AttachmentField {title: "Directory".to_string(), value: current_dir.display().to_string(), short: false};
    let hostname_field = AttachmentField {title: "Hostname".to_string(), value: whoami::hostname(), short: true};
    let user_field = AttachmentField {title: "User".to_string(), value: whoami::username(), short: true};
    let exec_field = AttachmentField {title: "executed at".to_string(), value: exec_at.to_string(), short: true};
    let elapsed_field = AttachmentField {title: "Elapsed time".to_string(), value: elapsed.to_string(), short: true};

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
        username: "Command result".to_string(),
        text: "<@".to_string() + &chairman + ">",
        attachments: vec!(attachment)
    };

    let client = reqwest::Client::new();
    let res = client.post(&webhook_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&webhook_req)
        .send()
        .await;

    match res {
        Ok(val) => println!("{:#?}", val),
        Err(err) => {
            println!("{}", err);
            process::exit(1)
        }
    }

    Ok(())
}
