use crate::model;

pub fn find_available_public_room<'st>(
    rooms: &'st [model::Room],
) -> Option<&'st model::Room> {
    rooms.iter().find(|room| {
        room.state == model::RoomState::Waiting
            && room.visibility == model::Visibility::Public
            && room.max_users != room.amount_of_users
    })
}

pub fn find_user_by_id<'st>(
    users: &'st [model::User],
    uid: &str,
) -> Option<&'st model::User> {
    users.iter().find(|user| user.id == uid)
}

pub fn find_room_by_id<'st>(
    rooms: &'st [model::Room],
    rid: &str,
) -> Option<&'st model::Room> {
    rooms.iter().find(|room| room.id == rid)
}

pub fn get_and_clone_users_in_room<'st>(
    users: &'st [model::User],
    rid: &str,
) -> Vec<model::User> {
    users
        .iter()
        .filter_map(|user| {
            if user.room_id == rid {
                return Some(user.clone());
            }

            None
        })
        .collect()
}
