use std::collections::HashMap;
use uuid::Uuid;
use crate::models::{Todo, TodoListFilter, TodoToggleAction};

#[derive(Debug, PartialEq, Eq)]
pub enum TodoRepoError{
    NotFound, 
}

#[derive(Default, Debug)]
pub struct TodoRepo{
    pub num_completed_items: u32, 
    pub num_active_items: u32,
    pub num_all_items: u32, 
    items: HashMap<Uuid, Todo>, 
}

impl TodoRepo{
    pub fn get(&self, id: &Uuid) -> Result<Todo, TodoRepoError>{
        self.items.get(id).cloned().ok_or(TodoRepoError::NotFound)
    }

    pub fn list(&self, filter: &TodoListFilter) -> Vec<Todo>{
        let mut todos = self.
            items.values()
            .filter(|item| match filter{
                TodoListFilter::Completed => item.is_completed, 
                TodoListFilter::Active => !item.is_completed, 
                TodoListFilter::All => true, 
            })
            .cloned()
            .collect::<Vec<_>>();
        
        
        todos.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        todos
            
    }

    pub fn create(&mut self, text: &str) -> Todo{

        let todo = Todo::new(text);
        self.items.insert(todo.id, todo.clone());
        self.num_active_items += 1;
        self.num_all_items += 1;

        todo
    }

    pub fn delete(&mut self, id: &Uuid) -> Result<(), TodoRepoError>{
        let item = self.items.remove(id).ok_or(TodoRepoError::NotFound)?;


        if item.is_completed{
            self.num_completed_items -= 1;
        } else{
            self.num_active_items -= 1;
        }

        self.num_all_items -= 1;

        Ok(())
    }

    pub fn update(
        &mut self, 
        id: &Uuid, 
        text: Option<String>, 
        is_completed: Option<bool>, 

    ) -> Result<Todo, TodoRepoError>{
        let todo = self.items.get_mut(id).ok_or(TodoRepoError::NotFound)?;

        if let Some(is_completed) = is_completed{
            todo.is_completed = is_completed;

            if todo.is_completed{
                self.num_completed_items += 1;
                self.num_active_items -= 1;

            } else{
                self.num_completed_items -= 1;
                self.num_active_items += 1;
            }
        }

        if let Some(text) = text{
            todo.text = text;
        }

        Ok(todo.clone())
    }

    pub fn delete_completed(&mut self){
        self.items.retain(|_, todo| !todo.is_completed);
        self.num_all_items -= self.num_completed_items;
        self.num_completed_items = 0;
        
    }

    pub fn toggle_completed(&mut self, action: &TodoToggleAction){
        let is_completed: bool;
        
        match action{
            TodoToggleAction::Uncheck => {
                self.num_completed_items = 0;
                self.num_active_items = self.num_all_items;

                is_completed = false;
            }
            TodoToggleAction::Check => {
                self.num_completed_items = self.num_all_items;
                self.num_active_items = 0;

                is_completed = true;
            }
        }

        for todo in self.items.values_mut(){
            todo.is_completed = is_completed;
        }

    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_get_non_existing_todo(){
        let repo = TodoRepo::default();
        let id = Uuid::new_v4();
        let result = repo.get(&id);
        assert_eq!(result, Err(TodoRepoError::NotFound));
    }
    #[test]
    fn test_get_existing_todo(){
        let todo = Todo::new("test");
        let id = Uuid::new_v4();

        let repo = TodoRepo{
            items: HashMap::from([(id, todo.clone())]), 
            ..Default::default()
        };

        let result = repo.get(&id);
        assert_eq!(result, Ok(todo));
    }

    #[test]
    fn test_list_empty_repo(){
        let repo = TodoRepo::default();
        let empty = Vec::new();

        let result_completed = repo.list(&TodoListFilter::Completed);
        let result_actived = repo.list(&TodoListFilter::Active);
        let result_all = repo.list(&TodoListFilter::All);

        assert_eq!(result_completed, empty);
        assert_eq!(result_actived, empty);
        assert_eq!(result_all, empty);

    }

    #[test]
    fn list_filled_repo_completed(){

        let mut todo_a = Todo::new("a");
        let mut todo_b = Todo::new("b");
        let todo_c = Todo::new("c");

        todo_a.is_completed = true;
        todo_b.is_completed = true;

        let completed = vec![todo_b.clone(), todo_a.clone()];
        let active = vec![todo_c.clone()];
        let all = [todo_c.clone(), todo_b.clone(), todo_a.clone()];


        let repo = TodoRepo{
            items: HashMap::from([
                    (Uuid::new_v4(), todo_a), 
                    (Uuid::new_v4(), todo_b), 
                    (Uuid::new_v4(), todo_c), 
                    ]),
                    ..Default::default()
        };

        let result_completed = repo.list(&TodoListFilter::Completed);
        let result_actived = repo.list(&TodoListFilter::Active);
        let result_all = repo.list(&TodoListFilter::All);

        assert_eq!(result_completed, completed);
        assert_eq!(result_actived, active);
        assert_eq!(result_all, all);


    }

    #[test]
    fn list_filled_repo_active(){

        let todo_a = Todo::new("a");
        let todo_b = Todo::new("b");
        let todo_c = Todo::new("c");

        let filled = [todo_c.clone(), todo_b.clone(), todo_a.clone()];
        let empty = Vec::new();

        let repo = TodoRepo{
            items: HashMap::from([
                    (Uuid::new_v4(), todo_a), 
                    (Uuid::new_v4(), todo_b), 
                    (Uuid::new_v4(), todo_c), 
                    ]),
                    ..Default::default()
        };

        let result_completed = repo.list(&TodoListFilter::Completed);
        let result_actived = repo.list(&TodoListFilter::Active);
        let result_all = repo.list(&TodoListFilter::All);

        assert_eq!(result_completed, empty);
        assert_eq!(result_actived, filled);
        assert_eq!(result_all, filled);


    }

    #[test]
    fn test_create_todo(){

        let mut repo = TodoRepo{
            items: HashMap::from([(Uuid::new_v4(), Todo::new("a"))]),
            num_completed_items: 0, 
            num_active_items: 1, 
            num_all_items: 1,
        };

        let result = repo.create("new");

        assert_eq!(result.text, "new".to_string());
        assert!(!result.is_completed);
        assert_eq!(repo.num_completed_items, 0);
        assert_eq!(repo.num_active_items, 2);
        assert_eq!(repo.num_all_items, 2);
    }

    #[test]
    fn test_delete_non_existing_todo(){

        let mut repo = TodoRepo::default();
        let id = Uuid::new_v4();
        let result = repo.delete(&id);

        assert_eq!(result, Err(TodoRepoError::NotFound));
    }

    #[test]
    fn test_delete_existing_todo(){
        let id = Uuid::new_v4();

        let mut repo = TodoRepo{
            items: HashMap::from([(id, Todo::new("a")), (Uuid::new_v4(), Todo::new("b"))]), 
            num_completed_items: 0, 
            num_active_items: 2, 
            num_all_items: 2,
        };

        let result = repo.delete(&id);

        assert_eq!(repo.num_completed_items, 0);
        assert_eq!(repo.num_active_items, 1);
        assert_eq!(repo.num_all_items, 1);

        assert_eq!(result, Ok(()));

    }

    #[test]
    fn test_update_non_existing_todo(){

        let mut repo = TodoRepo::default();
        let id = Uuid::new_v4();
        let result = repo.update(&id, None, None);

        assert_eq!(result, Err(TodoRepoError::NotFound));
    }

    #[test]
    fn test_update_text_existing_todo(){
        let todo = Todo::new("test");
        let id = Uuid::new_v4();

        let mut repo = TodoRepo{
            items: HashMap::from([(id, todo.clone())]), 
            num_completed_items: 0, 
            num_active_items: 1, 
            num_all_items: 1,
        };

        let result = repo.update(&id, Some("update".to_string()), None);

        assert!(result.is_ok());

        if let Ok(update) = result{
            assert_eq!(update.is_completed, todo.is_completed);
            assert_eq!(update.created_at, todo.created_at);
            assert_eq!(update.id, todo.id);
            assert_eq!(update.text, "update".to_string());
        }
    }

    #[test]
    fn test_update_is_completed_true_existing_todo(){
        let todo = Todo::new("test");
        let id = Uuid::new_v4();

        let mut repo = TodoRepo{
            items: HashMap::from([(id, todo.clone())]), 
            num_completed_items: 0, 
            num_active_items: 1, 
            num_all_items: 1, 
        };

        let result = repo.update(&id, None, Some(true));

        assert!(result.is_ok());

        if let Ok(update) = result{
            assert_eq!(update.created_at, todo.created_at);
            assert_eq!(update.text, todo.text);
            assert_eq!(update.id, todo.id);
            assert!(update.is_completed);
        }

        assert_eq!(repo.num_completed_items, 1);
        assert_eq!(repo.num_active_items, 0);
        assert_eq!(repo.num_all_items, 1);

    }

    #[test]
    fn test_update_is_completed_false_existing_todo(){
        let mut todo = Todo::new("test");
        let id = Uuid::new_v4();

        todo.is_completed = true;

        let mut repo = TodoRepo{
            items: HashMap::from([(id, todo.clone())]), 
            num_completed_items: 1, 
            num_active_items: 0, 
            num_all_items: 1, 
        };

        let result = repo.update(&id, None, Some(false));

        assert!(result.is_ok());

        if let Ok(update) = result{
            assert_eq!(update.created_at, todo.created_at);
            assert_eq!(update.text, todo.text);
            assert_eq!(update.id, todo.id);
            assert!(!update.is_completed);
        }

        assert_eq!(repo.num_completed_items, 0);
        assert_eq!(repo.num_active_items, 1);
        assert_eq!(repo.num_all_items, 1);

    }

    #[test]
    fn test_delete_completed_todos(){
        let mut todo_a = Todo::new("a");
        let mut todo_b = Todo::new("b");
        let todo_c = Todo::new("c");

        todo_a.is_completed = true;
        todo_b.is_completed = true;

        let active = vec![todo_c.clone()];

        let mut repo = TodoRepo{
            items: HashMap::from([
                    (Uuid::new_v4(), todo_a), 
                    (Uuid::new_v4(), todo_b), 
                    (Uuid::new_v4(), todo_c), 
                ]),
            num_completed_items: 2, 
            num_active_items: 1, 
            num_all_items: 3,
        };

        repo.delete_completed();

        assert_eq!(repo.items.into_values().collect::<Vec<_>>(), active);

        assert_eq!(repo.num_completed_items, 0);
        assert_eq!(repo.num_active_items, 1);
        assert_eq!(repo.num_all_items, 1);

    }

    #[test]
    fn test_toggle_check_completes_todos(){

        let mut todo_a = Todo::new("a");
        let mut todo_b = Todo::new("b");
        let todo_c = Todo::new("c");
        let id = Uuid::new_v4();

        todo_a.is_completed = true;
        todo_b.is_completed = true;

        let mut repo = TodoRepo{
            items: HashMap::from([
                    (Uuid::new_v4(), todo_a), 
                    (Uuid::new_v4(), todo_b), 
                    (id, todo_c), 
                ]),
            num_completed_items: 2, 
            num_active_items: 1, 
            num_all_items: 3,
        };

        repo.toggle_completed(&TodoToggleAction::Check);

        assert!(repo.items.get(&id).unwrap().is_completed);

        assert_eq!(repo.num_completed_items, 3);
        assert_eq!(repo.num_active_items, 0);
        assert_eq!(repo.num_all_items, 3);



    }

    #[test]
    fn test_toggle_uncheck_completes_todos(){

        let mut todo_a = Todo::new("a");
        let todo_b = Todo::new("b");
        let todo_c = Todo::new("c");
        let id = Uuid::new_v4();

        todo_a.is_completed = true;
        

        let mut repo = TodoRepo{
            items: HashMap::from([
                    (id, todo_a), 
                    (Uuid::new_v4(), todo_b), 
                    (Uuid::new_v4(), todo_c), 
                ]),
            num_completed_items: 1, 
            num_active_items: 2, 
            num_all_items: 3,
        };

        repo.toggle_completed(&TodoToggleAction::Uncheck);

        assert!(!repo.items.get(&id).unwrap().is_completed);

        assert_eq!(repo.num_completed_items, 0);
        assert_eq!(repo.num_active_items, 3);
        assert_eq!(repo.num_all_items, 3);



    }


}