use std::sync::Arc;

use crate::{asset_storage::AssetStorage, database::{IdentityDb, user_manager::{CreateUserData, UpdateProfileData, UserManager}}, service::identity::{CreateUserReq, Empty, FullProfile, FullProfileReq, NotifyUploadImageReq, Profile, Profiles, RemoveImageReq, UpdatePasswordReq, UpdateProfileReq, UploadImageReq, UploadImageRes, UserId, UserIds, VerifyPasswordReq, identity_server::Identity}, util};

pub mod identity {
    tonic::include_proto!("identity");
}

use tonic::{Request, Response, Status};

pub struct IdentityService {
    db: IdentityDb,
    asset_storage: Arc<AssetStorage>,
}

impl IdentityService {
    pub fn new(db: IdentityDb, asset_storage: Arc<AssetStorage>) -> Self {
        IdentityService { db, asset_storage }
    }
}

#[tonic::async_trait]
impl Identity for IdentityService {
    async fn get_profiles(&self, req: Request<UserIds>) -> Result<Response<Profiles>, Status> {
        let req = req.into_inner();
        let profiles = self.db
            .get_profiles(&req.user_ids)
            .await;
        let profiles = match profiles {
            Ok(v) => v.into_iter().map(|p| Profile {
                user_id: p.user_id,
                username: p.username,
                avatar_id: p.avatar_id, 
                banner_id: p.banner_id,
            }).collect(),
            Err(e) => return Err(Status::from_error(Box::new(e))),
        };

        Ok(Response::new(Profiles { profiles }))
    }

    async fn get_full_profile(&self, req: Request<FullProfileReq>) -> Result<Response<FullProfile>, Status> {
        let req = req.into_inner();
        let profile = self.db
            .get_full_profile(req.user_id, req.with_email)
            .await;
        let profile = match profile {
            Ok(res) => FullProfile {
                user_id: res.user_id,
                username: res.username,
                email: res.email,
                avatar_id: res.avatar_id,
                banner_id: res.banner_id,
                first_name: res.first_name,
                last_name: res.last_name,
                bio: res.bio,
                country: res.country,
                city: res.city,
            },
            Err(e) => return Err(Status::from_error(Box::new(e))),
        };

        Ok(Response::new(profile))
    }

    async fn get_basic_profile(&self, req: Request<UserId>) -> Result<Response<Profile>, Status> {
        let req = req.into_inner();
        let profile = self.db
            .get_basic_profile(req.user_id)
            .await;
        let profile = match profile {
            Ok(v) => Profile {
                user_id: v.user_id,
                username: v.username,
                avatar_id: v.avatar_id,
                banner_id: v.banner_id,
            },
            Err(e) => return Err(Status::from_error(Box::new(e))),
        };

        Ok(Response::new(profile))
    }

    async fn create_user(&self, req: Request<CreateUserReq>) -> Result<Response<UserId>, Status> {
        let req = req.into_inner();
        
        let data = CreateUserData {
            username: req.username,
            email: req.email,
            password: req.password,
            first_name: req.first_name,
            last_name: req.last_name,
        };
        let user_id = self.db.create_user(&data).await;
        let user_id = match user_id {
            Ok(v) => UserId { user_id: v },
            Err(e) => {
                if let Some(e) = e.downcast_ref::<sqlx::Error>() &&
                    let Some(de) = e.as_database_error() &&
                    de.is_unique_violation() {
                    return Err(Status::already_exists("Conflict"));
                }
                return Err(Status::from_error(e.into()));
            },
        };

        Ok(Response::new(user_id))
    }

    async fn delete_user(&self, req: Request<UserId>) -> Result<Response<Empty>, Status> {
        let req = req.into_inner();
        let res = self.db.delete_user(req.user_id).await;
        match res {
            Ok(_) => Ok(Response::new(Empty {})),
            Err(e) => Err(Status::from_error(Box::new(e))),
        }
    }

    async fn update_profile(&self, req: Request<UpdateProfileReq>) -> Result<Response<Empty>, Status> {
        let req = req.into_inner();

        let profile_data = UpdateProfileData {
            username: req.username,
            email: req.email,
            first_name: req.first_name,
            last_name: req.last_name,
            bio: req.bio,
            country: req.country,
            city: req.city,
        };
        let res = self.db
            .update_profile(req.user_id, &profile_data)
            .await;

        match res {
            Ok(_) => Ok(Response::new(Empty {})),
            Err(e) => {
                if let Some(de) = e.as_database_error() &&
                    de.is_unique_violation() {
                    return Err(Status::already_exists("Conflict"));
               }
                Err(Status::from_error(Box::new(e)))
            },
        }
    }

    async fn request_upload_image(
        &self,
        req: Request<UploadImageReq>,
    ) -> Result<Response<UploadImageRes>, Status> {
        let req = req.into_inner();

        let (upload_url, image_id) = self.asset_storage
            .get_upload_url(&req.mime_type)
            .await
            .map_err(|_| Status::internal("failed to get upload url"))?;

        Ok(Response::new(UploadImageRes {
            upload_url,
            image_id,
        }))
    }

    async fn notify_upload_avatar(
        &self,
        req: Request<NotifyUploadImageReq>,
    ) -> Result<Response<Empty>, Status> {
        let req = req.into_inner();

        let old_id = self.db
            .update_avatar(req.user_id, Some(&req.image_id))
            .await
            .map_err(|e| Status::from_error(Box::new(e)))?;

        util::assets::remove_asset(Arc::clone(&self.asset_storage), old_id);
        Ok(Response::new(Empty {}))
    }

    async fn remove_avatar(
        &self,
        req: Request<RemoveImageReq>,
    ) -> Result<Response<Empty>, Status> {
        let req = req.into_inner();

        let old_id = self.db
            .update_avatar(req.user_id, None)
            .await
            .map_err(|e| Status::from_error(Box::new(e)))?;

        util::assets::remove_asset(Arc::clone(&self.asset_storage), old_id);
        Ok(Response::new(Empty {}))
    }

    async fn notify_upload_banner(
        &self,
        req: Request<NotifyUploadImageReq>,
    ) -> Result<Response<Empty>, Status> {
        let req = req.into_inner();

        let old_id = self.db
            .update_banner(req.user_id, Some(&req.image_id))
            .await
            .map_err(|e| Status::from_error(Box::new(e)))?;

        util::assets::remove_asset(Arc::clone(&self.asset_storage), old_id);
        Ok(Response::new(Empty {}))
    }

    async fn remove_banner(
        &self,
        req: Request<RemoveImageReq>,
    ) -> Result<Response<Empty>, Status> {
        let req = req.into_inner();

        let old_id = self.db
            .update_banner(req.user_id, None)
            .await
            .map_err(|e| Status::from_error(Box::new(e)))?;

        util::assets::remove_asset(Arc::clone(&self.asset_storage), old_id);
        Ok(Response::new(Empty {}))
    }

    async fn update_password(&self, req: Request<UpdatePasswordReq>) -> Result<Response<Empty>, Status> {
        let req = req.into_inner();

        self.db
            .update_password(req.user_id, &req.old_password, &req.new_password)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(Empty {}))
    }

    async fn verify_password(&self, req: Request<VerifyPasswordReq>) -> Result<Response<UserId>, Status> {
        let req = req.into_inner();

        let user_id = self.db
            .verify_password(&req.username, &req.password)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(UserId { user_id }))
    }
}