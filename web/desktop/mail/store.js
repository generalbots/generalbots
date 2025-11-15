export function createMailStore() {
  return {
    selected: null,
    setSelected(id) {
      this.selected = id;
    }
  };
}

export const mailStore = createMailStore();
