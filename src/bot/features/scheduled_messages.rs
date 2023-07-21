use anyhow::anyhow;
use teloxide::types::ChatId;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    bot::utils::{get_random_media_info, send_media},
    database::{repository::AsyncRepository, types::CroneJob},
};

pub async fn start_messages_scheduling(bot: teloxide::Bot, mut repository: AsyncRepository) {
    let cron_jobs = match repository.cron_jobs().await {
        Err(e) => {
            log::error!("Failed to get cron jobs: '{e}'");
            return;
        }
        Ok(r) => r,
    };

    let sched = match JobScheduler::new().await {
        Err(e) => {
            log::error!("Failed to create job scheduler: '{e}'");
            return;
        }
        Ok(r) => r,
    };

    for cj in cron_jobs {
        let bot: teloxide::Bot = bot.clone();
        let repository = repository.clone();
        let pattern = cj.pattern.clone();

        let job = Job::new_async(pattern.as_str(), move |_, _| {
            let bot = bot.clone();
            let repository = repository.clone();
            let cj = cj.clone();

            Box::pin(async move {
                if let Err(e) = send_scheduled_message(bot, repository, cj).await {
                    log::error!("Failed to send scheduled message: '{e}'");
                }
            })
        });
        if let Err(e) = job {
            log::error!("Failed to create cron job: '{e}'");
            continue;
        }

        if let Err(e) = sched.add(job.unwrap()).await {
            log::error!("Failed to add cron job: '{e}'");
        }
    }

    if let Err(e) = sched.start().await {
        log::error!("Failed to start job scheduler: '{e}'");
    }
}

async fn send_scheduled_message(
    bot: teloxide::Bot,
    mut repository: AsyncRepository,
    cron_job: CroneJob,
) -> anyhow::Result<()> {
    if cron_job.chat_id.is_none() {
        return Err(anyhow!(
            "Sending messages to all discovered chats are not supported currently"
        ));
    }

    let media_infos = repository.media_info_by_cron_job_id(cron_job.id).await?;
    let media_info = get_random_media_info(&media_infos);
    if media_info.is_none() {
        return Err(anyhow!(
            "No media found for cron job '{}'",
            cron_job.pattern
        ));
    }

    send_media(
        media_info.unwrap(),
        &mut repository,
        bot,
        ChatId(cron_job.chat_id.unwrap()),
        None,
        cron_job.caption,
    )
    .await
}
