window.tasksApp = function tasksApp() {
  return {
    newTask: "",
    filter: "all",
    tasks: [],

    init() {
      const saved = localStorage.getItem("tasks");
      if (saved) {
        try {
          this.tasks = JSON.parse(saved);
        } catch (e) {
          console.error("Failed to load tasks:", e);
          this.tasks = [];
        }
      }
    },

    addTask() {
      if (this.newTask.trim() === "") return;

      this.tasks.push({
        id: Date.now(),
        text: this.newTask.trim(),
        completed: false,
        createdAt: new Date().toISOString(),
      });

      this.newTask = "";
      this.save();
    },

    toggleTask(id) {
      const task = this.tasks.find((t) => t.id === id);
      if (task) {
        task.completed = !task.completed;
        this.save();
      }
    },

    deleteTask(id) {
      this.tasks = this.tasks.filter((t) => t.id !== id);
      this.save();
    },

    clearCompleted() {
      this.tasks = this.tasks.filter((t) => !t.completed);
      this.save();
    },

    save() {
      try {
        localStorage.setItem("tasks", JSON.stringify(this.tasks));
      } catch (e) {
        console.error("Failed to save tasks:", e);
      }
    },

    get filteredTasks() {
      if (this.filter === "active") {
        return this.tasks.filter((t) => !t.completed);
      }
      if (this.filter === "completed") {
        return this.tasks.filter((t) => t.completed);
      }
      return this.tasks;
    },

    get activeTasks() {
      return this.tasks.filter((t) => !t.completed).length;
    },

    get completedTasks() {
      return this.tasks.filter((t) => t.completed).length;
    },
  };
};
