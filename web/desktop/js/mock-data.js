const mockUsers = [
  {
    id: 'user1',
    name: 'John Doe',
    email: 'john@example.com',
    avatar: '👨'
  },
  {
    id: 'user2', 
    name: 'Jane Smith',
    email: 'jane@example.com',
    avatar: '👩'
  }
];

const mockBots = [
  {
    id: 'default_bot',
    name: 'General Bot',
    description: 'Main assistant bot'
  }
];

const mockSessions = [
  {
    id: 'session1',
    title: 'First Chat',
    created_at: new Date().toISOString()
  },
  {
    id: 'session2',
    title: 'Project Discussion', 
    created_at: new Date(Date.now() - 86400000).toISOString()
  }
];

const mockAuthResponse = {
  user_id: mockUsers[0].id,
  session_id: mockSessions[0].id
};

const mockSuggestions = [
  { text: "What can you do?", context: "capabilities" },
  { text: "Show my files", context: "drive" },
  { text: "Create a task", context: "tasks" }
];
