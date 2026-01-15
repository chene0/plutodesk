"use client";

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

interface SessionResponse {
  id: string;
  name: string;
  folder_id: string;
  course_id: string;
  subject_id: string;
  folder_name: string;
  course_name: string;
  subject_name: string;
  created_at: string;
  last_used: string;
}

interface Folder {
  id: string;
  name: string;
}

interface Course {
  id: string;
  name: string;
}

interface Subject {
  id: string;
  name: string;
}

export function SessionModal() {
  const [isOpen, setIsOpen] = useState(false);
  const [sessions, setSessions] = useState<SessionResponse[]>([]);
  const [activeSession, setActiveSession] = useState<SessionResponse | null>(null);
  const [showNewSessionForm, setShowNewSessionForm] = useState(false);

  // New session form state
  const [folderInput, setFolderInput] = useState("");
  const [courseInput, setCourseInput] = useState("");
  const [subjectInput, setSubjectInput] = useState("");

  // Dropdowns data
  const [folders, setFolders] = useState<Folder[]>([]);
  const [courses, setCourses] = useState<Course[]>([]);
  const [subjects, setSubjects] = useState<Subject[]>([]);

  const [selectedFolderId, setSelectedFolderId] = useState<string | null>(null);
  const [selectedCourseId, setSelectedCourseId] = useState<string | null>(null);
  const [selectedSubjectId, setSelectedSubjectId] = useState<string | null>(null);

  // "Create new" mode state
  const [isCreatingNewFolder, setIsCreatingNewFolder] = useState(false);
  const [isCreatingNewCourse, setIsCreatingNewCourse] = useState(false);
  const [isCreatingNewSubject, setIsCreatingNewSubject] = useState(false);
  const [newFolderName, setNewFolderName] = useState("");
  const [newCourseName, setNewCourseName] = useState("");
  const [newSubjectName, setNewSubjectName] = useState("");

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Default user ID (for MVP)
  const DEFAULT_USER_ID = "00000000-0000-0000-0000-000000000000";

  useEffect(() => {
    // Listen for open-session-modal event from tray
    const unlisten = listen("open-session-modal", () => {
      setIsOpen(true);
      loadSessions();
      loadActiveSession();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (isOpen) {
      loadFolders();
    }
  }, [isOpen]);

  useEffect(() => {
    if (selectedFolderId) {
      loadCourses(selectedFolderId);
    } else {
      setCourses([]);
      setSubjects([]);
    }
  }, [selectedFolderId]);

  useEffect(() => {
    if (selectedCourseId) {
      loadSubjects(selectedCourseId);
    } else {
      setSubjects([]);
    }
  }, [selectedCourseId]);

  const loadSessions = async () => {
    try {
      const result = await invoke<SessionResponse[]>("get_all_sessions");
      setSessions(result);
    } catch (err) {
      console.error("Failed to load sessions:", err);
      setError("Failed to load sessions");
    }
  };

  const loadActiveSession = async () => {
    try {
      const result = await invoke<SessionResponse | null>("get_active_session");
      setActiveSession(result);
    } catch (err) {
      console.error("Failed to load active session:", err);
    }
  };

  const loadFolders = async () => {
    try {
      const result = await invoke<string>("get_folders_by_user", {
        userId: DEFAULT_USER_ID,
      });
      const parsed = JSON.parse(result) as Folder[];
      setFolders(parsed);
    } catch (err) {
      console.error("Failed to load folders:", err);
    }
  };

  const loadCourses = async (folderId: string) => {
    try {
      const result = await invoke<string>("get_courses_by_folder", {
        folderId,
      });
      const parsed = JSON.parse(result) as Course[];
      setCourses(parsed);
    } catch (err) {
      console.error("Failed to load courses:", err);
    }
  };

  const loadSubjects = async (courseId: string) => {
    try {
      const result = await invoke<string>("get_subjects_by_course", {
        courseId,
      });
      const parsed = JSON.parse(result) as Subject[];
      setSubjects(parsed);
    } catch (err) {
      console.error("Failed to load subjects:", err);
    }
  };

  const handleStartSession = async (sessionId: string) => {
    setLoading(true);
    setError(null);
    try {
      await invoke("start_session", { sessionId });
      await loadActiveSession();
      setIsOpen(false);
    } catch (err) {
      console.error("Failed to start session:", err);
      setError("Failed to start session");
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteSession = async (sessionId: string) => {
    if (!confirm("Are you sure you want to delete this session?")) {
      return;
    }

    setLoading(true);
    setError(null);
    try {
      await invoke("delete_session", { sessionId });
      await loadSessions();
      await loadActiveSession();
    } catch (err) {
      console.error("Failed to delete session:", err);
      setError("Failed to delete session");
    } finally {
      setLoading(false);
    }
  };

  const validateNewItemName = (name: string, existingItems: Array<{name: string}>, itemType: string): boolean => {
    if (!name.trim()) {
      setError(`${itemType} name cannot be empty`);
      return false;
    }
    
    const duplicate = existingItems.find(
      (item) => item.name.toLowerCase() === name.trim().toLowerCase()
    );
    
    if (duplicate) {
      setError(`A ${itemType.toLowerCase()} named "${name}" already exists. Please select it from the dropdown or choose a different name.`);
      return false;
    }
    
    return true;
  };

  const resetFormState = () => {
    setShowNewSessionForm(false);
    setFolderInput("");
    setCourseInput("");
    setSubjectInput("");
    setIsCreatingNewFolder(false);
    setIsCreatingNewCourse(false);
    setIsCreatingNewSubject(false);
    setNewFolderName("");
    setNewCourseName("");
    setNewSubjectName("");
    setSelectedFolderId(null);
    setSelectedCourseId(null);
    setSelectedSubjectId(null);
    setError(null);
  };

  const handleCreateSession = async () => {
    // Determine final names (either from dropdown selection or new input)
    const finalFolderName = isCreatingNewFolder ? newFolderName : folderInput;
    const finalCourseName = isCreatingNewCourse ? newCourseName : courseInput;
    const finalSubjectName = isCreatingNewSubject ? newSubjectName : subjectInput;

    // Validate all fields are filled
    if (!finalFolderName.trim() || !finalCourseName.trim() || !finalSubjectName.trim()) {
      setError("Folder, course, and subject are required");
      return;
    }

    // Validate no duplicates for new items
    if (isCreatingNewFolder && !validateNewItemName(newFolderName, folders, "Folder")) {
      return;
    }
    if (isCreatingNewCourse && !validateNewItemName(newCourseName, courses, "Course")) {
      return;
    }
    if (isCreatingNewSubject && !validateNewItemName(newSubjectName, subjects, "Subject")) {
      return;
    }

    setLoading(true);
    setError(null);
    try {
      await invoke("create_and_start_session", {
        request: {
          folder_name: finalFolderName,
          course_name: finalCourseName,
          subject_name: finalSubjectName,
        },
      });
      await loadSessions();
      await loadActiveSession();
      resetFormState();
      setIsOpen(false);
    } catch (err) {
      console.error("Failed to create session:", err);
      setError(typeof err === "string" ? err : "Failed to create session");
    } finally {
      setLoading(false);
    }
  };

  const handleFolderChange = (value: string) => {
    if (value === "CREATE_NEW") {
      setIsCreatingNewFolder(true);
      setFolderInput("");
      setSelectedFolderId(null);
      setCourseInput("");
      setSelectedCourseId(null);
      setSubjectInput("");
      setSelectedSubjectId(null);
      setCourses([]);
      setSubjects([]);
    } else {
      setIsCreatingNewFolder(false);
      const folder = folders.find((f) => f.id === value);
      if (folder) {
        setFolderInput(folder.name);
        setSelectedFolderId(folder.id);
        setCourseInput("");
        setSelectedCourseId(null);
        setSubjectInput("");
        setSelectedSubjectId(null);
        setSubjects([]);
      }
    }
  };

  const handleCourseChange = (value: string) => {
    if (value === "CREATE_NEW") {
      setIsCreatingNewCourse(true);
      setCourseInput("");
      setSelectedCourseId(null);
      setSubjectInput("");
      setSelectedSubjectId(null);
      setSubjects([]);
    } else {
      setIsCreatingNewCourse(false);
      const course = courses.find((c) => c.id === value);
      if (course) {
        setCourseInput(course.name);
        setSelectedCourseId(course.id);
        setSubjectInput("");
        setSelectedSubjectId(null);
      }
    }
  };

  const handleSubjectChange = (value: string) => {
    if (value === "CREATE_NEW") {
      setIsCreatingNewSubject(true);
      setSubjectInput("");
      setSelectedSubjectId(null);
    } else {
      setIsCreatingNewSubject(false);
      const subject = subjects.find((s) => s.id === value);
      if (subject) {
        setSubjectInput(subject.name);
        setSelectedSubjectId(subject.id);
      }
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl p-6 max-w-2xl w-full max-h-[80vh] overflow-y-auto">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-2xl font-bold">Session Management</h2>
          <button
            onClick={() => setIsOpen(false)}
            className="text-gray-500 hover:text-gray-700"
          >
            ✕
          </button>
        </div>

        {error && (
          <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
            {error}
          </div>
        )}

        {activeSession && (
          <div className="bg-blue-50 border border-blue-200 rounded p-4 mb-4">
            <h3 className="font-semibold text-blue-900 mb-2">Active Session</h3>
            <p className="text-blue-800">
              <strong>{activeSession.name}</strong>
            </p>
            <p className="text-sm text-blue-600">
              {activeSession.folder_name} → {activeSession.course_name} →{" "}
              {activeSession.subject_name}
            </p>
          </div>
        )}

        {!showNewSessionForm ? (
          <>
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-2">Saved Sessions</h3>
              {sessions.length === 0 ? (
                <p className="text-gray-500">No saved sessions</p>
              ) : (
                <div className="space-y-2">
                  {sessions.map((session) => (
                    <div
                      key={session.id}
                      className="border rounded p-3 flex justify-between items-center"
                    >
                      <div>
                        <p className="font-medium">{session.name}</p>
                        <p className="text-sm text-gray-600">
                          {session.folder_name} → {session.course_name} →{" "}
                          {session.subject_name}
                        </p>
                      </div>
                      <div className="flex gap-2">
                        <button
                          onClick={() => handleStartSession(session.id)}
                          disabled={loading || activeSession?.id === session.id}
                          className="bg-blue-500 text-white px-3 py-1 rounded hover:bg-blue-600 disabled:bg-gray-300"
                        >
                          {activeSession?.id === session.id ? "Active" : "Start"}
                        </button>
                        <button
                          onClick={() => handleDeleteSession(session.id)}
                          disabled={loading}
                          className="bg-red-500 text-white px-3 py-1 rounded hover:bg-red-600 disabled:bg-gray-300"
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <button
              onClick={() => setShowNewSessionForm(true)}
              className="w-full bg-green-500 text-white py-2 rounded hover:bg-green-600"
            >
              + New Session
            </button>
          </>
        ) : (
          <div className="space-y-4">
            <h3 className="text-lg font-semibold">Create New Session</h3>
            <p className="text-sm text-gray-600">
              Session name will be auto-generated from folder/course/subject
            </p>

            <div>
              <label htmlFor="session-folder" className="block text-sm font-medium mb-1">
                Folder
              </label>
              {!isCreatingNewFolder ? (
                <select
                  id="session-folder"
                  value={selectedFolderId || ""}
                  onChange={(e) => handleFolderChange(e.target.value)}
                  className="w-full border rounded px-3 py-2"
                >
                  <option value="">Select a folder...</option>
                  {folders.map((folder) => (
                    <option key={folder.id} value={folder.id}>
                      {folder.name}
                    </option>
                  ))}
                  <option value="CREATE_NEW">+ Create new folder...</option>
                </select>
              ) : (
                <div className="space-y-2">
                  <input
                    id="session-folder"
                    type="text"
                    value={newFolderName}
                    onChange={(e) => setNewFolderName(e.target.value)}
                    className="w-full border rounded px-3 py-2"
                    placeholder="Enter new folder name"
                    autoFocus
                  />
                  <button
                    type="button"
                    onClick={() => {
                      setIsCreatingNewFolder(false);
                      setNewFolderName("");
                    }}
                    className="text-sm text-gray-600 hover:text-gray-800"
                  >
                    ← Back to folder selection
                  </button>
                </div>
              )}
            </div>

            <div>
              <label htmlFor="session-course" className="block text-sm font-medium mb-1">
                Course
              </label>
              {!isCreatingNewCourse ? (
                <select
                  id="session-course"
                  value={selectedCourseId || ""}
                  onChange={(e) => handleCourseChange(e.target.value)}
                  className="w-full border rounded px-3 py-2"
                  disabled={!selectedFolderId && !isCreatingNewFolder}
                >
                  <option value="">Select a course...</option>
                  {courses.map((course) => (
                    <option key={course.id} value={course.id}>
                      {course.name}
                    </option>
                  ))}
                  <option value="CREATE_NEW">+ Create new course...</option>
                </select>
              ) : (
                <div className="space-y-2">
                  <input
                    id="session-course"
                    type="text"
                    value={newCourseName}
                    onChange={(e) => setNewCourseName(e.target.value)}
                    className="w-full border rounded px-3 py-2"
                    placeholder="Enter new course name"
                    autoFocus
                  />
                  <button
                    type="button"
                    onClick={() => {
                      setIsCreatingNewCourse(false);
                      setNewCourseName("");
                    }}
                    className="text-sm text-gray-600 hover:text-gray-800"
                  >
                    ← Back to course selection
                  </button>
                </div>
              )}
            </div>

            <div>
              <label htmlFor="session-subject" className="block text-sm font-medium mb-1">
                Subject
              </label>
              {!isCreatingNewSubject ? (
                <select
                  id="session-subject"
                  value={selectedSubjectId || ""}
                  onChange={(e) => handleSubjectChange(e.target.value)}
                  className="w-full border rounded px-3 py-2"
                  disabled={!selectedCourseId && !isCreatingNewCourse}
                >
                  <option value="">Select a subject...</option>
                  {subjects.map((subject) => (
                    <option key={subject.id} value={subject.id}>
                      {subject.name}
                    </option>
                  ))}
                  <option value="CREATE_NEW">+ Create new subject...</option>
                </select>
              ) : (
                <div className="space-y-2">
                  <input
                    id="session-subject"
                    type="text"
                    value={newSubjectName}
                    onChange={(e) => setNewSubjectName(e.target.value)}
                    className="w-full border rounded px-3 py-2"
                    placeholder="Enter new subject name"
                    autoFocus
                  />
                  <button
                    type="button"
                    onClick={() => {
                      setIsCreatingNewSubject(false);
                      setNewSubjectName("");
                    }}
                    className="text-sm text-gray-600 hover:text-gray-800"
                  >
                    ← Back to subject selection
                  </button>
                </div>
              )}
            </div>

            <div className="flex gap-2">
              <button
                onClick={handleCreateSession}
                disabled={loading}
                className="flex-1 bg-blue-500 text-white py-2 rounded hover:bg-blue-600 disabled:bg-gray-300"
              >
                Create & Start Session
              </button>
              <button
                onClick={resetFormState}
                className="flex-1 bg-gray-300 text-gray-700 py-2 rounded hover:bg-gray-400"
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

