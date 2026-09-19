use crate::{avatar_storage::AvatarStorage, database::{IdentityDb, user_manager::{CreateUserData, ProfileMetadata, UserManager}}, service::identity::{CreateUserReq, Empty, FullProfile, Profile, Profiles, UpdateAvatarReq, UpdatePasswordReq, UpdateProfileReq, UserId, UserIds, VerifyPasswordReq}};

pub mod identity {
    tonic::include_proto!("identity");
}

use identity::identity_server::Identity;
use tonic::{Request, Response, Status};

pub struct IdentityService {
    db: IdentityDb,
    avatar_storage: AvatarStorage,
}

impl IdentityService {
    pub fn new(db: IdentityDb, avatar_storage: AvatarStorage) -> Self {
        IdentityService { db, avatar_storage }
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
                avatar_url: p.avatar_url, 
            }).collect(),
            Err(e) => return Err(Status::from_error(Box::new(e))),
        };

        Ok(Response::new(Profiles { profiles }))
    }

    async fn get_full_profile(&self, req: Request<UserId>) -> Result<Response<FullProfile>, Status> {
        let req = req.into_inner();
        let profile = self.db
            .get_full_profile(req.user_id)
            .await;
        let profile = match profile {
            Ok(v) => FullProfile {
                user_id: v.user_id,
                username: v.username,
                avatar_url: v.avatar_url,
                first_name: v.first_name,
                last_name: v.last_name,
                bio: v.bio,
                country: v.country,
                city: v.city,
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
                avatar_url: v.avatar_url,
            },
            Err(e) => return Err(Status::from_error(Box::new(e))),
        };

        Ok(Response::new(profile))
    }

    async fn create_user(&self, req: Request<CreateUserReq>) -> Result<Response<UserId>, Status> {
        let req = req.into_inner();
        
        let metadata = CreateUserData {
            username: req.username,
            email: req.email,
            password: req.password,
            first_name: req.first_name,
            last_name: req.last_name,
        };
        let user_id = self.db.create_user(&metadata).await;
        let user_id = match user_id {
            Ok(v) => UserId { user_id: v },
            Err(e) => return Err(Status::from_error(e.into())),
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

        let profile_metadata = ProfileMetadata {
            username: req.username,
            email: req.email,
            first_name: req.first_name,
            last_name: req.last_name,
            bio: req.bio,
            country: req.country,
            city: req.city,
        };
        let res = self.db
            .update_profile(req.user_id, &profile_metadata)
            .await;

        match res {
            Ok(_) => Ok(Response::new(Empty {})),
            Err(e) => Err(Status::from_error(Box::new(e))),
        }
    }

    async fn update_avatar(&self, req: Request<UpdateAvatarReq>) -> Result<Response<Empty>, Status> {
        let req = req.into_inner();
        
        let key = self.avatar_storage
            .upload_avatar(&req.image)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        let old_key = self.db
            .update_avatar(req.user_id, &key)
            .await
            .map_err(|e| Status::from_error(Box::new(e)));
        let old_key = match old_key {
            Ok(v) => v,
            Err(e) => {
                let _ = self.avatar_storage
                    .remove_avatar(&key)
                    .await;
                return Err(Status::from_error(Box::new(e)));
            }
        };

        if let Some(key) = old_key {
            let _ = self.avatar_storage
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