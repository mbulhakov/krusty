use anyhow::anyhow;
use teloxide::types::ChatId;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    bot::utils::{get_random_media_info, send_media},
    database::{repository::AsyncRepository, types::CroneJob},
};

pub async fn create_scheduler(
    bot: teloxide::Bot,
    mut repository: AsyncRepository,
) -> anyhow::Result<JobScheduler> {
    let cron_jobs = repository.cron_jobs().await?;

    let sched = JobScheduler::new().await?;

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
            log::error!("Failed to create cron job '{pattern}': '{e}'");
            continue;
        }

        if let Err(e) = sched.add(job.unwrap()).await {
            log::error!("Failed to add cron job '{pattern}': '{e}'");
        }
    }

    Ok(sched)
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
