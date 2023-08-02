use std::collections::{HashMap, HashSet};

use anyhow::anyhow;
use teloxide::{types::ChatId, Bot};
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

use crate::{
    bot::utils::{get_random_media_info, send_media},
    database::{repository::AsyncRepository, types::CroneJob},
};

pub struct Scheduler {
    pub(in crate::bot::features::schedule) inner: JobScheduler,
    bot: Bot,
    ids_to_uids: HashMap<i32, Uuid>,
    repository: AsyncRepository,
}

impl Scheduler {
    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.sync().await?;
        self.inner.start().await?;

        Ok(())
    }

    pub async fn sync(&mut self) -> anyhow::Result<()> {
        let cron_jobs = self.repository.cron_jobs().await?;
        let job_ids: HashSet<_> = cron_jobs.iter().map(|x| x.id).collect();

        let current_job_ids: HashSet<_> = self.ids_to_uids.keys().cloned().collect();

        let obsolete_job_ids = current_job_ids.difference(&job_ids);
        for id in obsolete_job_ids {
            let uuid = self.ids_to_uids.get(id).unwrap();
            if let Err(e) = self.inner.remove(uuid).await {
                log::error!("Failed to remove cron job with uuid '{uuid}': '{e}'");
            }
        }

        let job_ids_to_add: HashSet<_> = job_ids.difference(&current_job_ids).collect();
        for cj in cron_jobs
            .into_iter()
            .filter(|x| job_ids_to_add.contains(&x.id))
        {
            let bot: teloxide::Bot = self.bot.clone();
            let repository = self.repository.clone();
            let pattern = cj.pattern.clone();
            let id = cj.id;

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

            match self.inner.add(job.unwrap()).await {
                Ok(uuid) => {
                    self.ids_to_uids.insert(id, uuid);
                }
                Err(e) => log::error!("Failed to add cron job '{pattern}': '{e}'"),
            };
        }

        Ok(())
    }

    pub(in crate::bot::features::schedule) async fn new(
        bot: Bot,
        repository: AsyncRepository,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            inner: JobScheduler::new().await?,
            bot,
            ids_to_uids: HashMap::new(),
            repository,
        })
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
