/**
 * Learn Module - JavaScript Controller
 * Learning Management System for General Bots
 */

// State management
const LearnState = {
  courses: [],
  myCourses: [],
  mandatoryAssignments: [],
  certificates: [],
  categories: [],
  currentCourse: null,
  currentLesson: null,
  currentQuiz: null,
  quizAnswers: {},
  quizTimer: null,
  quizTimeRemaining: 0,
  currentQuestionIndex: 0,
  filters: {
    category: "all",
    difficulty: ["beginner", "intermediate", "advanced"],
    search: "",
    sort: "recent",
  },
  pagination: {
    offset: 0,
    limit: 12,
    hasMore: true,
  },
  userStats: {
    coursesCompleted: 0,
    coursesInProgress: 0,
    certificates: 0,
    timeSpent: 0,
  },
};

// API Base URL
const LEARN_API = "/api/learn";

// ============================================================================
// INITIALIZATION
// ============================================================================

document.addEventListener("DOMContentLoaded", () => {
  initLearn();
});

function initLearn() {
  loadUserStats();
  loadCategories();
  loadCourses();
  loadMyCourses();
  loadMandatoryAssignments();
  loadCertificates();
  loadRecommendations();
  bindEvents();
}

function bindEvents() {
  // Search input
  const searchInput = document.getElementById("searchCourses");
  if (searchInput) {
    let searchTimeout;
    searchInput.addEventListener("input", (e) => {
      clearTimeout(searchTimeout);
      searchTimeout = setTimeout(() => {
        LearnState.filters.search = e.target.value;
        LearnState.pagination.offset = 0;
        loadCourses();
      }, 300);
    });
  }

  // Sort select
  const sortSelect = document.getElementById("sortCourses");
  if (sortSelect) {
    sortSelect.addEventListener("change", (e) => {
      LearnState.filters.sort = e.target.value;
      LearnState.pagination.offset = 0;
      loadCourses();
    });
  }

  // Category filters
  document.querySelectorAll(".category-item").forEach((item) => {
    item.addEventListener("click", () => {
      document
        .querySelectorAll(".category-item")
        .forEach((i) => i.classList.remove("active"));
      item.classList.add("active");
      LearnState.filters.category = item.dataset.category;
      LearnState.pagination.offset = 0;
      loadCourses();
    });
  });

  // Difficulty filters
  document.querySelectorAll("[data-difficulty]").forEach((checkbox) => {
    checkbox.addEventListener("change", () => {
      LearnState.filters.difficulty = Array.from(
        document.querySelectorAll("[data-difficulty]:checked"),
      ).map((cb) => cb.dataset.difficulty);
      LearnState.pagination.offset = 0;
      loadCourses();
    });
  });

  // Close modals on background click
  document.querySelectorAll(".modal").forEach((modal) => {
    modal.addEventListener("click", (e) => {
      if (e.target === modal) {
        modal.classList.add("hidden");
      }
    });
  });

  // Keyboard shortcuts
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      closeAllModals();
    }
  });
}

// ============================================================================
// API CALLS
// ============================================================================

async function apiCall(endpoint, options = {}) {
  try {
    const response = await fetch(`${LEARN_API}${endpoint}`, {
      headers: {
        "Content-Type": "application/json",
        ...options.headers,
      },
      ...options,
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error || "API request failed");
    }

    return data;
  } catch (error) {
    console.error("API Error:", error);
    showNotification("error", "Erro ao carregar dados. Tente novamente.");
    throw error;
  }
}

async function loadUserStats() {
  try {
    const response = await apiCall("/stats/user");
    if (response.success) {
      LearnState.userStats = response.data;
      updateUserStatsUI();
    }
  } catch (error) {
    // Use mock data if API fails
    updateUserStatsUI();
  }
}

async function loadCategories() {
  try {
    const response = await apiCall("/categories");
    if (response.success) {
      LearnState.categories = response.data;
      updateCategoryCounts();
    }
  } catch (error) {
    // Categories loaded from HTML
  }
}

async function loadCourses() {
  try {
    const params = new URLSearchParams({
      limit: LearnState.pagination.limit,
      offset: LearnState.pagination.offset,
    });

    if (LearnState.filters.category !== "all") {
      params.append("category", LearnState.filters.category);
    }

    if (LearnState.filters.search) {
      params.append("search", LearnState.filters.search);
    }

    if (LearnState.filters.difficulty.length < 3) {
      params.append("difficulty", LearnState.filters.difficulty.join(","));
    }

    const response = await apiCall(`/courses?${params}`);

    if (response.success) {
      if (LearnState.pagination.offset === 0) {
        LearnState.courses = response.data;
      } else {
        LearnState.courses = [...LearnState.courses, ...response.data];
      }

      LearnState.pagination.hasMore =
        response.data.length >= LearnState.pagination.limit;
      renderCourses();
    }
  } catch (error) {
    // Render mock courses for demo
    renderMockCourses();
  }
}

async function loadMyCourses() {
  try {
    const response = await apiCall("/progress");
    if (response.success) {
      LearnState.myCourses = response.data;
      renderMyCourses();
    }
  } catch (error) {
    renderMockMyCourses();
  }
}

async function loadMandatoryAssignments() {
  try {
    const response = await apiCall("/assignments/pending");
    if (response.success) {
      LearnState.mandatoryAssignments = response.data;
      renderMandatoryAssignments();
      updateMandatoryAlert();
    }
  } catch (error) {
    renderMockMandatory();
  }
}

async function loadCertificates() {
  try {
    const response = await apiCall("/certificates");
    if (response.success) {
      LearnState.certificates = response.data;
      renderCertificates();
    }
  } catch (error) {
    renderMockCertificates();
  }
}

async function loadRecommendations() {
  try {
    const response = await apiCall("/recommendations");
    if (response.success) {
      renderRecommendations(response.data);
    }
  } catch (error) {
    renderMockRecommendations();
  }
}

async function loadCourseDetail(courseId) {
  try {
    const response = await apiCall(`/courses/${courseId}`);
    if (response.success) {
      LearnState.currentCourse = response.data;
      renderCourseModal(response.data);
    }
  } catch (error) {
    showMockCourseDetail(courseId);
  }
}

async function startCourseAPI(courseId) {
  try {
    const response = await apiCall(`/progress/${courseId}/start`, {
      method: "POST",
    });
    if (response.success) {
      showNotification("success", "Curso iniciado com sucesso!");
      loadMyCourses();
      return response.data;
    }
  } catch (error) {
    showNotification("error", "Erro ao iniciar o curso.");
  }
}

async function completeLessonAPI(lessonId) {
  try {
    const response = await apiCall(`/progress/${lessonId}/complete`, {
      method: "POST",
    });
    if (response.success) {
      showNotification("success", "Aula concluída!");
      return response.data;
    }
  } catch (error) {
    showNotification("error", "Erro ao marcar aula como concluída.");
  }
}

async function submitQuizAPI(courseId, answers) {
  try {
    const response = await apiCall(`/courses/${courseId}/quiz`, {
      method: "POST",
      body: JSON.stringify({ answers }),
    });
    if (response.success) {
      return response.data;
    }
  } catch (error) {
    showNotification("error", "Erro ao enviar respostas.");
  }
}

// ============================================================================
// UI RENDERING
// ============================================================================

function updateUserStatsUI() {
  document.getElementById("statCoursesCompleted").textContent =
    LearnState.userStats.courses_completed || 0;
  document.getElementById("statCoursesInProgress").textContent =
    LearnState.userStats.courses_in_progress || 0;
  document.getElementById("statCertificates").textContent =
    LearnState.userStats.certificates_earned || 0;
  document.getElementById("statTimeSpent").textContent =
    `${LearnState.userStats.total_time_spent_hours || 0}h`;
}

function updateCategoryCounts() {
  // Update category counts based on courses
  const counts = {};
  LearnState.courses.forEach((course) => {
    counts[course.category] = (counts[course.category] || 0) + 1;
  });

  document.getElementById("countAll").textContent = LearnState.courses.length;
  document.getElementById("countMandatory").textContent =
    LearnState.mandatoryAssignments.length;
}

function renderCourses() {
  const grid = document.getElementById("coursesGrid");
  const countLabel = document.getElementById("coursesCountLabel");

  if (!grid) return;

  if (LearnState.pagination.offset === 0) {
    grid.innerHTML = "";
  }

  if (LearnState.courses.length === 0) {
    grid.innerHTML = `
            <div class="empty-state">
                <span>📚</span>
                <h3>Nenhum curso encontrado</h3>
                <p>Tente ajustar os filtros de busca.</p>
            </div>
        `;
    countLabel.textContent = "0 cursos";
    return;
  }

  LearnState.courses.forEach((course) => {
    grid.appendChild(createCourseCard(course));
  });

  countLabel.textContent = `${LearnState.courses.length} cursos`;

  // Show/hide load more button
  const loadMore = document.getElementById("loadMore");
  if (loadMore) {
    loadMore.style.display = LearnState.pagination.hasMore ? "block" : "none";
  }
}

function createCourseCard(course) {
  const card = document.createElement("div");
  card.className = "course-card";
  card.onclick = () => openCourseModal(course.id);

  const difficultyClass = (course.difficulty || "beginner").toLowerCase();
  const progress = course.user_progress || 0;

  card.innerHTML = `
        <div class="course-thumbnail">
            ${
              course.thumbnail_url
                ? `<img src="${course.thumbnail_url}" alt="${course.title}">`
                : `<span class="placeholder-icon">📖</span>`
            }
            ${course.is_mandatory ? '<span class="course-mandatory-badge">Obrigatório</span>' : ""}
            ${progress > 0 ? `<span class="course-progress-badge">${progress}%</span>` : ""}
        </div>
        <div class="course-content">
            <h3 class="course-title">${escapeHtml(course.title)}</h3>
            <div class="course-meta">
                <span class="difficulty-badge ${difficultyClass}">${formatDifficulty(course.difficulty)}</span>
                <span>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"></circle>
                        <polyline points="12 6 12 12 16 14"></polyline>
                    </svg>
                    ${formatDuration(course.duration_minutes)}
                </span>
            </div>
            ${
              progress > 0
                ? `
                <div class="course-progress">
                    <div class="progress-bar">
                        <div class="progress-fill" style="width: ${progress}%"></div>
                    </div>
                    <span class="progress-text">${progress}% completo</span>
                </div>
            `
                : ""
            }
        </div>
    `;

  return card;
}

function renderMyCourses() {
  const continueLearning = document.getElementById("continueLearning");
  const completedCourses = document.getElementById("completedCourses");

  if (!continueLearning || !completedCourses) return;

  const inProgress = LearnState.myCourses.filter(
    (c) => c.status === "in_progress",
  );
  const completed = LearnState.myCourses.filter(
    (c) => c.status === "completed",
  );

  // Update badge
  document.getElementById("myCoursesCount").textContent = inProgress.length;

  // Render in-progress courses
  if (inProgress.length === 0) {
    continueLearning.innerHTML = `
            <div class="empty-state-small">
                <span>📚</span>
                <p>Nenhum curso em andamento</p>
            </div>
        `;
  } else {
    continueLearning.innerHTML = inProgress
      .map((course) => createCourseListItem(course))
      .join("");
  }

  // Render completed courses
  if (completed.length === 0) {
    completedCourses.innerHTML = `
            <div class="empty-state-small">
                <span>✅</span>
                <p>Nenhum curso concluído ainda</p>
            </div>
        `;
  } else {
    completedCourses.innerHTML = completed
      .map((course) => createCourseListItem(course, true))
      .join("");
  }
}

function createCourseListItem(course, isCompleted = false) {
  return `
        <div class="course-list-item" onclick="openCourseModal('${course.course_id}')">
            <div class="course-list-thumbnail">
                <span class="placeholder-icon">📖</span>
            </div>
            <div class="course-list-info">
                <h4>${escapeHtml(course.course_title || course.title || "Curso")}</h4>
                <div class="course-meta">
                    <span>${formatDuration(course.duration_minutes || 30)}</span>
                </div>
            </div>
            <div class="course-list-progress">
                <div class="progress-bar">
                    <div class="progress-fill" style="width: ${course.completion_percentage || (isCompleted ? 100 : 0)}%"></div>
                </div>
                <span class="progress-text">${course.completion_percentage || (isCompleted ? 100 : 0)}% completo</span>
            </div>
            <div class="course-list-action">
                <button class="btn-primary-sm">
                    ${isCompleted ? "Revisar" : "Continuar"}
                </button>
            </div>
        </div>
    `;
}

function renderMandatoryAssignments() {
  const list = document.getElementById("mandatoryList");
  const badge = document.getElementById("mandatoryCount");

  if (!list) return;

  badge.textContent = LearnState.mandatoryAssignments.length;

  if (LearnState.mandatoryAssignments.length === 0) {
    list.innerHTML = `
            <div class="empty-state">
                <span>🎉</span>
                <h3>Tudo em dia!</h3>
                <p>Você não possui treinamentos obrigatórios pendentes.</p>
            </div>
        `;
    return;
  }

  list.innerHTML = LearnState.mandatoryAssignments
    .map((assignment) => {
      const isOverdue =
        assignment.due_date && new Date(assignment.due_date) < new Date();
      const daysUntilDue = assignment.due_date
        ? Math.ceil(
            (new Date(assignment.due_date) - new Date()) /
              (1000 * 60 * 60 * 24),
          )
        : null;
      const isUrgent =
        daysUntilDue !== null && daysUntilDue <= 7 && daysUntilDue > 0;

      return `
            <div class="mandatory-item ${isOverdue ? "overdue" : ""} ${isUrgent ? "urgent" : ""}"
                 onclick="openCourseModal('${assignment.course_id}')">
                <div class="mandatory-icon">
                    ${isOverdue ? "⚠️" : isUrgent ? "⏰" : "📋"}
                </div>
                <div class="mandatory-info">
                    <h4>${escapeHtml(assignment.course_title || "Treinamento Obrigatório")}</h4>
                    <div class="mandatory-due ${isOverdue ? "overdue" : ""} ${isUrgent ? "urgent" : ""}">
                        ${
                          isOverdue
                            ? "<span>⚠️ Prazo vencido!</span>"
                            : daysUntilDue !== null
                              ? `<span>Prazo: ${daysUntilDue} dias</span>`
                              : "<span>Sem prazo definido</span>"
                        }
                    </div>
                </div>
                <button class="btn-primary">
                    ${isOverdue ? "Iniciar Agora" : "Começar"}
                </button>
            </div>
        `;
    })
    .join("");
}

function updateMandatoryAlert() {
  const alert = document.getElementById("mandatoryAlert");
  const alertText = document.getElementById("mandatoryAlertText");

  if (!alert) return;

  const overdueCount = LearnState.mandatoryAssignments.filter(
    (a) => a.due_date && new Date(a.due_date) < new Date(),
  ).length;

  const urgentCount = LearnState.mandatoryAssignments.filter((a) => {
    if (!a.due_date) return false;
    const days = Math.ceil(
      (new Date(a.due_date) - new Date()) / (1000 * 60 * 60 * 24),
    );
    return days > 0 && days <= 7;
  }).length;

  if (overdueCount > 0 || urgentCount > 0) {
    alert.style.display = "flex";
    if (overdueCount > 0) {
      alertText.textContent = `Você possui ${overdueCount} treinamento(s) com prazo vencido!`;
    } else {
      alertText.textContent = `Você possui ${urgentCount} treinamento(s) com prazo próximo.`;
    }
  } else {
    alert.style.display = "none";
  }
}

function renderCertificates() {
  const grid = document.getElementById("certificatesGrid");
  const preview = document.getElementById("certificatesPreview");

  if (!grid) return;

  if (LearnState.certificates.length === 0) {
    grid.innerHTML = `
            <div class="empty-state">
                <span>🏆</span>
                <h3>Nenhum certificado ainda</h3>
                <p>Complete seus cursos para ganhar certificados.</p>
            </div>
        `;

    if (preview) {
      preview.innerHTML = `
                <div class="empty-state-small">
                    <span>🏆</span>
                    <p>Nenhum certificado ainda</p>
                </div>
            `;
    }
    return;
  }

  grid.innerHTML = LearnState.certificates
    .map(
      (cert) => `
        <div class="certificate-card" onclick="openCertificateModal('${cert.id}')">
            <div class="certificate-card-header">
                <span class="cert-icon">🎓</span>
                <h4>${escapeHtml(cert.course_title || "Curso Concluído")}</h4>
            </div>
            <div class="certificate-card-body">
                <span class="cert-score">${cert.score}%</span>
                <span class="cert-date">${formatDate(cert.issued_at)}</span>
            </div>
            <div class="certificate-card-footer">
                <button class="btn-secondary" onclick="event.stopPropagation(); downloadCertificateById('${cert.id}')">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                        <polyline points="7 10 12 15 17 10"></polyline>
                        <line x1="12" y1="15" x2="12" y2="3"></line>
                    </svg>
                    Baixar
                </button>
                <button class="btn-link" onclick="event.stopPropagation(); shareCertificate('${cert.verification_code}')">
                    Compartilhar
                </button>
            </div>
        </div>
    `,
    )
    .join("");

  // Update sidebar preview
  if (preview) {
    preview.innerHTML = LearnState.certificates
      .slice(0, 3)
      .map(
        (cert) => `
            <div class="cert-preview-item" onclick="openCertificateModal('${cert.id}')">
                <span class="cert-icon">🎓</span>
                <div class="cert-info">
                    <div class="cert-title">${escapeHtml(cert.course_title || "Curso")}</div>
                    <div class="cert-date">${formatDate(cert.issued_at)}</div>
                </div>
            </div>
        `,
      )
      .join("");
  }
}

function renderRecommendations(courses) {
  const carousel = document.getElementById("recommendedCourses");
  if (!carousel) return;

  if (!courses || courses.length === 0) {
    carousel.innerHTML =
      '<p class="text-secondary">Explore o catálogo para encontrar cursos.</p>';
    return;
  }

  carousel.innerHTML = courses
    .slice(0, 6)
    .map((course) => {
      const card = createCourseCard(course);
      card.style.minWidth = "280px";
      return card.outerHTML;
    })
    .join("");
}

// ============================================================================
// MODALS
// ============================================================================

function openCourseModal(courseId) {
  loadCourseDetail(courseId);
  document.getElementById("courseModal").classList.remove("hidden");
}

function closeCourseModal() {
  document.getElementById("courseModal").classList.add("hidden");
  LearnState.currentCourse = null;
}

function renderCourseModal(data) {
  const { course, lessons, quiz } = data;

  document.getElementById("modalCourseTitle").textContent = course.title;
  document.getElementById("modalDescription").textContent =
    course.description || "Sem descrição disponível.";
  document.getElementById("modalDifficulty").textContent = formatDifficulty(
    course.difficulty,
  );
  document.getElementById("modalDifficulty").className =
    `difficulty-badge ${(course.difficulty || "beginner").toLowerCase()}`;
  document.getElementById("modalDuration").querySelector("span").textContent =
    formatDuration(course.duration_minutes);
  document
    .getElementById("modalLessonsCount")
    .querySelector("span").textContent = `${lessons?.length || 0} aulas`;

  // Render lessons
  const lessonsList = document.getElementById("modalLessonsList");
  if (lessons && lessons.length > 0) {
    lessonsList.innerHTML = lessons
      .map(
        (lesson, index) => `
            <div class="lesson-item ${lesson.is_completed ? "completed" : ""}"
                 onclick="openLesson('${lesson.id}', ${index})">
                <span class="lesson-number">${lesson.is_completed ? "✓" : index + 1}</span>
                <div class="lesson-info">
                    <h5>${escapeHtml(lesson.title)}</h5>
                    <span>${formatDuration(lesson.duration_minutes)}</span>
                </div>
                <span class="lesson-action">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polygon points="5 3 19 12 5 21 5 3"></polygon>
                    </svg>
                </span>
            </div>
        `,
      )
      .join("");
  } else {
    lessonsList.innerHTML =
      '<p class="text-secondary">Nenhuma aula disponível.</p>';
  }

  // Render quiz section
  const quizSection = document.getElementById("modalQuizSection");
  if (quiz) {
    quizSection.style.display = "block";
    const questions =
      typeof quiz.questions === "string"
        ? JSON.parse(quiz.questions)
        : quiz.questions || [];
    document.getElementById("modalQuizQuestions").textContent =
      `${questions.length} questões`;
    document.getElementById("modalQuizTime").textContent =
      quiz.time_limit_minutes ? `${quiz.time_limit_minutes} min` : "Sem limite";
    document.getElementById("modalQuizPassing").textContent =
      `${quiz.passing_score}% para aprovação`;
    LearnState.currentQuiz = quiz;
  } else {
    quizSection.style.display = "none";
  }

  // Update button text based on progress
  const startBtn = document.getElementById("startCourseBtn");
  if (data.user_progress) {
    const progress = data.user_progress;
    if (progress.status === "completed") {
      startBtn.innerHTML = `
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="1 4 1 10 7 10"></polyline>
                    <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"></path>
                </svg>
                <span>Revisar Curso</span>
            `;
    } else if (progress.status === "in_progress") {
      startBtn.innerHTML = `
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polygon points="5 3 19 12 5 21 5 3"></polygon>
                </svg>
                <span>Continuar</span>
            `;
    }

    // Show progress
    document.getElementById("modalProgress").style.display = "block";
    document.getElementById("modalProgressFill").style.width =
      `${progress.completion_percentage || 0}%`;
    document.getElementById("modalProgressText").textContent =
      `${progress.completion_percentage || 0}% completo`;
  } else {
    document.getElementById("modalProgress").style.display = "none";
    startBtn.innerHTML = `
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polygon points="5 3 19 12 5 21 5 3"></polygon>
            </svg>
            <span>Iniciar Curso</span>
        `;
  }
}

function startCourse() {
  if (!LearnState.currentCourse) return;

  const course = LearnState.currentCourse.course || LearnState.currentCourse;
  const lessons = LearnState.currentCourse.lessons || [];

  // Start course via API
  startCourseAPI(course.id);

  // Open first lesson
  if (lessons.length > 0) {
    openLesson(lessons[0].id, 0);
  } else {
    showNotification("info", "Este curso ainda não possui aulas.");
  }
}

function openLesson(lessonId, index) {
  const lessons = LearnState.currentCourse?.lessons || [];
  const lesson = lessons.find((l) => l.id === lessonId) || lessons[index];

  if (!lesson) {
    showNotification("error", "Aula não encontrada.");
    return;
  }

  LearnState.currentLesson = lesson;
  LearnState.currentLessonIndex = index;

  // Close course modal, open lesson modal
  closeCourseModal();
  document.getElementById("lessonModal").classList.remove("hidden");

  // Update lesson UI
  document.getElementById("lessonTitle").textContent = lesson.title;
  document.getElementById("lessonNavTitle").textContent =
    `Aula ${index + 1} de ${lessons.length}`;

  // Render content based on type
  const contentDiv = document.getElementById("lessonContent");
  if (lesson.video_url) {
    contentDiv.innerHTML = `
            <div class="video-container">
                <iframe src="${lesson.video_url}" frameborder="0" allowfullscreen></iframe>
            </div>
            <div class="lesson-text">
                ${lesson.content || ""}
            </div>
        `;
  } else {
    contentDiv.innerHTML = `
            <div class="lesson-text">
                ${lesson.content || "<p>Conteúdo da aula será exibido aqui.</p>"}
            </div>
        `;
  }

  // Update sidebar list
  const sidebar = document.getElementById("lessonListSidebar");
  sidebar.innerHTML = lessons
    .map(
      (l, i) => `
        <div class="lesson-item ${l.is_completed ? "completed" : ""} ${l.id === lesson.id ? "active" : ""}"
             onclick="openLesson('${l.id}', ${i})">
            <span class="lesson-number">${l.is_completed ? "✓" : i + 1}</span>
            <div class="lesson-info">
                <h5>${escapeHtml(l.title)}</h5>
            </div>
        </div>
    `,
    )
    .join("");

  // Update navigation buttons
  document.getElementById("prevLessonBtn").disabled = index === 0;
  document.getElementById("nextLessonBtn").disabled =
    index >= lessons.length - 1;

  // Update progress
  const progress = ((index + 1) / lessons.length) * 100;
  document.getElementById("lessonProgressFill").style.width = `${progress}%`;
}

function closeLessonModal() {
  document.getElementById("lessonModal").classList.add("hidden");
  LearnState.currentLesson = null;

  // Reopen course modal
  if (LearnState.currentCourse) {
    openCourseModal(LearnState.currentCourse.id);
  }
}

function toggleLearnSidebar() {
  const sidebar = document.querySelector(".learn-sidebar");
  if (sidebar) {
    sidebar.classList.toggle("collapsed");
  }
}

function switchTab(tabId) {
  document.querySelectorAll(".tab-btn, .learn-tab-btn").forEach((btn) => {
    btn.classList.remove("active");
    if (
      btn.dataset.tab === tabId ||
      btn.getAttribute("onclick")?.includes(tabId)
    ) {
      btn.classList.add("active");
    }
  });

  document
    .querySelectorAll(".tab-content, .learn-tab-content")
    .forEach((content) => {
      content.classList.add("hidden");
      if (content.id === tabId || content.id === `${tabId}-tab`) {
        content.classList.remove("hidden");
      }
    });
}

function showAllCertificates() {
  switchTab("certificates");
}

function loadMoreCourses() {
  const currentCount = document.querySelectorAll(".course-card").length;
  fetch(`/api/learn/courses?offset=${currentCount}&limit=12`)
    .then((r) => r.json())
    .then((data) => {
      const grid = document.querySelector(".courses-grid");
      if (grid && data.courses) {
        data.courses.forEach((course) => {
          grid.insertAdjacentHTML("beforeend", createCourseCard(course));
        });
      }
      if (!data.hasMore) {
        const btn = document.querySelector('[onclick="loadMoreCourses()"]');
        if (btn) btn.style.display = "none";
      }
    })
    .catch((err) => console.error("Error loading more courses:", err));
}

function startQuiz() {
  if (!LearnState.currentCourse) return;

  LearnState.quizState = {
    questions: LearnState.currentCourse.quiz || [],
    currentIndex: 0,
    answers: {},
    startTime: Date.now(),
  };

  document.getElementById("courseModal").classList.add("hidden");
  document.getElementById("quizModal").classList.remove("hidden");
  renderQuizQuestion();
}

function renderQuizQuestion() {
  const { questions, currentIndex } = LearnState.quizState;
  if (!questions || questions.length === 0) return;

  const question = questions[currentIndex];
  const container = document.getElementById("quizQuestionContainer");

  container.innerHTML = `
        <div class="quiz-question">
            <div class="question-number">Question ${currentIndex + 1} of ${questions.length}</div>
            <h3>${question.text}</h3>
            <div class="quiz-options">
                ${question.options
                  .map(
                    (opt, i) => `
                    <label class="quiz-option ${LearnState.quizState.answers[currentIndex] === i ? "selected" : ""}">
                        <input type="radio" name="q${currentIndex}" value="${i}"
                            ${LearnState.quizState.answers[currentIndex] === i ? "checked" : ""}
                            onchange="selectAnswer(${i})">
                        <span>${opt}</span>
                    </label>
                `,
                  )
                  .join("")}
            </div>
        </div>
    `;

  document.getElementById("quizProgress").textContent =
    `${currentIndex + 1}/${questions.length}`;
  document.getElementById("quizProgressFill").style.width =
    `${((currentIndex + 1) / questions.length) * 100}%`;
}

function selectAnswer(index) {
  LearnState.quizState.answers[LearnState.quizState.currentIndex] = index;
}

function prevQuestion() {
  if (LearnState.quizState.currentIndex > 0) {
    LearnState.quizState.currentIndex--;
    renderQuizQuestion();
  }
}

function nextQuestion() {
  if (
    LearnState.quizState.currentIndex <
    LearnState.quizState.questions.length - 1
  ) {
    LearnState.quizState.currentIndex++;
    renderQuizQuestion();
  }
}

function submitQuiz() {
  const { questions, answers, startTime } = LearnState.quizState;
  let correct = 0;

  questions.forEach((q, i) => {
    if (answers[i] === q.correctIndex) correct++;
  });

  const score = Math.round((correct / questions.length) * 100);
  const passed = score >= 70;
  const duration = Math.round((Date.now() - startTime) / 1000);

  LearnState.quizResult = {
    score,
    correct,
    total: questions.length,
    passed,
    duration,
  };

  document.getElementById("quizModal").classList.add("hidden");
  document.getElementById("quizResultModal").classList.remove("hidden");

  document.getElementById("quizScore").textContent = `${score}%`;
  document.getElementById("quizCorrect").textContent =
    `${correct}/${questions.length}`;
  document.getElementById("quizStatus").textContent = passed
    ? "Passed!"
    : "Not Passed";
  document.getElementById("quizStatus").className = passed
    ? "status-passed"
    : "status-failed";

  if (passed && LearnState.currentCourse) {
    fetch(`/api/learn/courses/${LearnState.currentCourse.id}/complete`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ score, duration }),
    }).catch((err) => console.error("Error completing course:", err));
  }
}

function closeQuizResult() {
  document.getElementById("quizResultModal").classList.add("hidden");
  LearnState.quizState = null;
  LearnState.quizResult = null;
  renderMyCourses();
  renderCertificates();
}

function reviewAnswers() {
  document.getElementById("quizResultModal").classList.add("hidden");
  document.getElementById("quizModal").classList.remove("hidden");
  LearnState.quizState.currentIndex = 0;
  renderQuizQuestion();
}

function confirmExitQuiz() {
  if (confirm("Are you sure you want to exit? Your progress will be lost.")) {
    document.getElementById("quizModal").classList.add("hidden");
    LearnState.quizState = null;
    if (LearnState.currentCourse) {
      openCourseModal(LearnState.currentCourse.id);
    }
  }
}

function downloadCertificate() {
  if (!LearnState.currentCourse) return;
  window.open(
    `/api/learn/certificates/${LearnState.currentCourse.id}/download`,
    "_blank",
  );
}

function downloadCertificateById(certId) {
  window.open(`/api/learn/certificates/${certId}/download`, "_blank");
}

function closeCertificateModal() {
  document.getElementById("certificateModal").classList.add("hidden");
}

function prevLesson() {
  if (!LearnState.currentCourse || !LearnState.currentLesson) return;
  const lessons = LearnState.currentCourse.lessons || [];
  const currentIndex = lessons.findIndex(
    (l) => l.id === LearnState.currentLesson.id,
  );
  if (currentIndex > 0) {
    openLesson(lessons[currentIndex - 1].id, currentIndex - 1);
  }
}

function nextLesson() {
  if (!LearnState.currentCourse || !LearnState.currentLesson) return;
  const lessons = LearnState.currentCourse.lessons || [];
  const currentIndex = lessons.findIndex(
    (l) => l.id === LearnState.currentLesson.id,
  );
  if (currentIndex < lessons.length - 1) {
    openLesson(lessons[currentIndex + 1].id, currentIndex + 1);
  }
}

function completeLesson() {
  if (!LearnState.currentLesson || !LearnState.currentCourse) return;

  fetch(`/api/learn/lessons/${LearnState.currentLesson.id}/complete`, {
    method: "POST",
  })
    .then(() => {
      LearnState.currentLesson.completed = true;
      closeLessonModal();
    })
    .catch((err) => console.error("Error completing lesson:", err));
}

// Export functions to window
window.toggleLearnSidebar = toggleLearnSidebar;
window.showAllCertificates = showAllCertificates;
window.loadMoreCourses = loadMoreCourses;
window.startQuiz = startQuiz;
window.prevQuestion = prevQuestion;
window.nextQuestion = nextQuestion;
window.submitQuiz = submitQuiz;
window.closeQuizResult = closeQuizResult;
window.reviewAnswers = reviewAnswers;
window.confirmExitQuiz = confirmExitQuiz;
window.downloadCertificate = downloadCertificate;
window.downloadCertificateById = downloadCertificateById;
window.closeCertificateModal = closeCertificateModal;
window.prevLesson = prevLesson;
window.nextLesson = nextLesson;
window.completeLesson = completeLesson;
window.selectAnswer = selectAnswer;
window.switchTab = switchTab;
window.openCourseModal = openCourseModal;
window.closeCourseModal = closeCourseModal;
window.startCourse = startCourse;
window.openLesson = openLesson;
window.closeLessonModal = closeLessonModal;
