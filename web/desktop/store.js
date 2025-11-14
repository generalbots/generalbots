document.addEventListener('alpine:init', () => {
  Alpine.store('todo', {
    title: 'Todo',
    items: [],
    nextId: 1,
    
    addTodo(text) {
      if (!text.trim()) return;
      
      this.items.push({
        id: this.nextId,
        title: text.trim(),
        done: false
      });
      this.nextId++;
    },
    
    toggleTodo(id) {
      this.items = this.items.map(item => 
        item.id === id ? { ...item, done: !item.done } : item
      );
    },
    
    removeTodo(id) {
      this.items = this.items.filter(item => item.id !== id);
    },
    
    clearCompleted() {
      this.items = this.items.filter(item => !item.done);
    }
  });
});
