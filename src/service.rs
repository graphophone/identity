use crate::{database::{IdentityDb, user_manager::{CreateUserData, UpdateProfileData, UserManager}}, image_storage::ImageStorage, service::identity::{CreateUserReq, Empty, FullProfile, FullProfileReq, Profile, Profiles, UpdateAvatarReq, UpdateBannerReq, UpdatePasswordReq, UpdateProfileReq, UserId, UserIds, VerifyPasswordReq}};

pub mod identity {
    tonic::include_proto!("identity");
}

use identity::identity_server::Identity;
use tonic::{Request, Response, Status};

pub struct IdentityService {
    db: IdentityDb,
    image_storage: ImageStorage,
}

impl IdentityService {
    pub fn new(db: IdentityDb, image_storage: ImageStorage) -> Self {
        IdentityService { db, image_storage }
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
                avatar_key: p.avatar_key, 
                banner_key: p.banner_key,
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
                avatar_key: res.avatar_key,
                banner_key: res.banner_key,
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
                avatar_key: v.avatar_key,
                banner_key: v.banner_key,
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

    async fn update_avatar(&self, req: Request<UpdateAvatarReq>) -> Result<Response<Empty>, Status> {
        let req = req.into_inner();

        let mut key = None;
        if let Some(avatar) = req.avatar {
            let key_str = self.image_storage
                .upload_avatar(&avatar)
                .await
                .map_err(|e| Status::from_error(e.into()))?;
            key = Some(key_str);
        }
        

        let old_key: Result<Option<String>, Status> = self.db
            .update_avatar(req.user_id, key.as_ref().map(|v| v.as_str()))
            .await
            .map_err(|e| Status::from_error(Box::new(e)));
        let old_key = match old_key {
            Ok(v) => v,
            Err(e) => {
                if let Some(key) = key {
                    let _ = self.image_storage
                        .remove_avatar(&key)
                        .await;
                }
                return Err(Status::from_error(Box::new(e)));
            }
        };

        if let Some(key) = old_key {
            let _ = self.image_storage
                .remove_avatar(&key)
                .await;
        }

        Ok(Response::new(Empty {}))
    }

    async fn update_banner(&self, req: Request<UpdateBannerReq>) -> Result<Response<Empty>, Status> {
        let req = req.into_inner();

        let mut key = None;
        if let Some(banner) = req.banner {
            let key_str = self.image_storage
                .upload_banner(&banner)
                .await
                .map_err(|e| Status::from_error(e.into()))?;
            key = Some(key_str);
        }
        

        let old_key: Result<Option<String>, Status> = self.db
            .update_banner(req.user_id, key.as_ref().map(|v| v.as_str()))
            .await
            .map_err(|e| Status::from_error(Box::new(e)));
        let old_key = match old_key {
            Ok(v) => v,
            Err(e) => {
                if let Some(key) = key {
                    let _ = self.image_storage
                        .remove_banner(&key)
                        .await;
                }
                return Err(Status::from_error(Box::new(e)));
            }
        };

        if let Some(key) = old_key {
            let _ = self.image_storage
                .remove_avatar(&key)
                .await;
        }

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