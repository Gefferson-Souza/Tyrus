class TodoItem {
  id: number;
  title: string;
  completed: boolean;

  constructor(id: number, title: string) {
    this.id = id;
    this.title = title;
    this.completed = false;
  }
}

class TodoList {
  items: TodoItem[];

  constructor() {
    this.items = [];
  }

  add(title: string) {
    let id = this.items.length + 1;
    this.items.push(new TodoItem(id, title));
  }

  // Use map to immutably update list (Generator friendly)
  toggle(id: number) {
    this.items = this.items.map((item) => {
      if (item.id == id) {
        let newItem = new TodoItem(item.id, item.title);
        if (!item.completed) {
          newItem.completed = true;
        }
        return newItem;
      } else {
        return item;
      }
    });
  }

  filter(completed: boolean): TodoItem[] {
    return this.items.filter((i) => i.completed == completed);
  }
}

const list = new TodoList();
list.add("Buy milk");
list.add("Walk dog");
list.toggle(1);

console.log(JSON.stringify(list.items));
