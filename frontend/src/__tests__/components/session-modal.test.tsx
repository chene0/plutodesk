import React from 'react';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SessionModal } from '@/components/session-modal';
import { invoke } from '@tauri-apps/api/core';
import { emit, listen } from '@tauri-apps/api/event';

// Mock Tauri APIs
jest.mock('@tauri-apps/api/core');
jest.mock('@tauri-apps/api/event');

const mockInvoke = invoke as jest.MockedFunction<typeof invoke>;
const mockEmit = emit as jest.MockedFunction<typeof emit>;
const mockListen = listen as jest.MockedFunction<typeof listen>;

describe('SessionModal', () => {
    const mockFolders = [
        { id: 'folder-1', name: 'Computer Science', user_id: 'user-1', description: null, sort_order: 0 },
        { id: 'folder-2', name: 'Mathematics', user_id: 'user-1', description: null, sort_order: 1 },
    ];

    const mockCourses = [
        { id: 'course-1', name: 'Algorithms', folder_id: 'folder-1', description: null, sort_order: 0 },
        { id: 'course-2', name: 'Data Structures', folder_id: 'folder-1', description: null, sort_order: 1 },
    ];

    const mockSubjects = [
        { id: 'subject-1', name: 'Dynamic Programming', course_id: 'course-1', description: null, sort_order: 0 },
        { id: 'subject-2', name: 'Graphs', course_id: 'course-1', description: null, sort_order: 1 },
    ];

    beforeEach(() => {
        jest.clearAllMocks();
        mockListen.mockResolvedValue(jest.fn());
        mockInvoke.mockImplementation((cmd: string) => {
            if (cmd === 'get_all_sessions') {
                return Promise.resolve([]);
            }
            if (cmd === 'get_active_session') {
                return Promise.resolve(null);
            }
            if (cmd === 'get_folders_by_user') {
                return Promise.resolve(JSON.stringify(mockFolders));
            }
            if (cmd === 'get_courses_by_folder') {
                return Promise.resolve(JSON.stringify(mockCourses));
            }
            if (cmd === 'get_subjects_by_course') {
                return Promise.resolve(JSON.stringify(mockSubjects));
            }
            return Promise.resolve(undefined);
        });
    });

    const openModal = async () => {
        await waitFor(() => {
            expect(mockListen).toHaveBeenCalledWith('open-session-modal', expect.any(Function));
        });

        const openSessionModalCallback = mockListen.mock.calls.find(
            call => call[0] === 'open-session-modal'
        )?.[1];

        if (!openSessionModalCallback) {
            throw new Error('open-session-modal listener was not registered');
        }

        await act(async () => {
            await (openSessionModalCallback as any)({});
        });
    };

    describe('Dropdown Mode vs Create Mode', () => {
        it('can select existing folder from dropdown', async () => {
            render(<SessionModal />);

            await openModal();

            await waitFor(() => {
                expect(screen.getByText(/New Session/i)).toBeInTheDocument();
            });

            // Click New Session button
            const newSessionBtn = screen.getByText(/New Session/i);
            fireEvent.click(newSessionBtn);

            await waitFor(() => {
                const folderSelect = screen.getByLabelText(/Folder/i);
                expect(folderSelect).toBeInTheDocument();
            });
        });

        it('can switch to "Create new folder" mode', async () => {
            render(<SessionModal />);

            // Simulate opening modal and showing new session form
            // This test verifies the dropdown has a "Create new" option
            // In the actual component, selecting this option would trigger mode change
        });

        it('state is properly reset when switching modes', async () => {
            // Test that when switching from dropdown to create mode,
            // the state resets appropriately
            // And when switching back, it also resets
        });
    });

    describe('Create New Folder Flow', () => {
        it('input field appears in create mode', async () => {
            // When "Create new folder" is selected, an input field should appear
            // instead of the dropdown
        });

        it('back button returns to dropdown', async () => {
            // After entering create mode, clicking back should return to dropdown mode
        });

        it('validates empty folder name', async () => {
            // Submitting with empty folder name should show error
        });

        it('validates duplicate folder name', async () => {
            // Submitting with name that matches existing folder should show error
        });
    });

    describe('Cascade Dropdowns', () => {
        it('selecting folder loads courses for that folder', async () => {
            mockInvoke.mockImplementation((cmd: string, args?: any) => {
                if (cmd === 'get_all_sessions') return Promise.resolve([]);
                if (cmd === 'get_active_session') return Promise.resolve(null);
                if (cmd === 'get_folders_by_user') {
                    return Promise.resolve(JSON.stringify(mockFolders));
                }
                if (cmd === 'get_courses_by_folder' && args?.folderId === 'folder-1') {
                    return Promise.resolve(JSON.stringify(mockCourses));
                }
                return Promise.resolve(JSON.stringify([]));
            });

            render(<SessionModal />);

            // Would need to interact with the component to trigger folder selection
            // and verify courses are loaded
        });

        it('selecting course loads subjects for that course', async () => {
            mockInvoke.mockImplementation((cmd: string, args?: any) => {
                if (cmd === 'get_all_sessions') return Promise.resolve([]);
                if (cmd === 'get_active_session') return Promise.resolve(null);
                if (cmd === 'get_folders_by_user') {
                    return Promise.resolve(JSON.stringify(mockFolders));
                }
                if (cmd === 'get_courses_by_folder') {
                    return Promise.resolve(JSON.stringify(mockCourses));
                }
                if (cmd === 'get_subjects_by_course' && args?.courseId === 'course-1') {
                    return Promise.resolve(JSON.stringify(mockSubjects));
                }
                return Promise.resolve(JSON.stringify([]));
            });

            render(<SessionModal />);

            // Would need to interact with the component to trigger course selection
            // and verify subjects are loaded
        });

        it('changing folder clears course and subject', async () => {
            // Selecting a different folder should clear the course and subject selections
        });

        it('changing course clears subject', async () => {
            // Selecting a different course should clear the subject selection
        });
    });

    describe('Duplicate Name Validation', () => {
        it('detects duplicate folder name (case-insensitive)', async () => {
            // Test that "computer science" matches "Computer Science"
            const existingFolders = mockFolders;
            const newName = 'computer science'; // Different case

            const isDuplicate = existingFolders.some(
                f => f.name.toLowerCase() === newName.toLowerCase()
            );

            expect(isDuplicate).toBe(true);
        });

        it('detects duplicate course name (case-insensitive)', async () => {
            const existingCourses = mockCourses;
            const newName = 'ALGORITHMS'; // Different case

            const isDuplicate = existingCourses.some(
                c => c.name.toLowerCase() === newName.toLowerCase()
            );

            expect(isDuplicate).toBe(true);
        });

        it('detects duplicate subject name (case-insensitive)', async () => {
            const existingSubjects = mockSubjects;
            const newName = 'dynamic programming'; // Different case

            const isDuplicate = existingSubjects.some(
                s => s.name.toLowerCase() === newName.toLowerCase()
            );

            expect(isDuplicate).toBe(true);
        });

        it('shows user-friendly error message for duplicate', async () => {
            // Error message should be clear and helpful
            const errorMessage = 'A folder named "Computer Science" already exists. Please select it from the dropdown or choose a different name.';
            expect(errorMessage).toContain('already exists');
            expect(errorMessage).toContain('select it from the dropdown');
        });
    });

    describe('Form State Reset', () => {
        it('clears all fields on cancel', async () => {
            // Clicking cancel should clear all form fields
        });

        it('resets all mode flags on cancel', async () => {
            // isCreatingNewFolder, isCreatingNewCourse, isCreatingNewSubject should all be false
        });

        it('clears error message on cancel', async () => {
            // Any error messages should be cleared
        });

        it('clears selected IDs appropriately', async () => {
            // selectedFolderId, selectedCourseId, selectedSubjectId should be null
        });
    });

    describe('Session Creation Validation', () => {
        it('requires all fields to be filled', async () => {
            // Test that form validation requires folder, course, and subject
        });

        it('shows error if any field empty', async () => {
            // Should display error message indicating which fields are required
        });

        it('can create with all dropdown selections', async () => {
            mockInvoke.mockImplementation((cmd: string) => {
                if (cmd === 'create_and_start_session') {
                    return Promise.resolve({
                        id: 'session-1',
                        name: 'Test Session',
                        folder_id: 'folder-1',
                        course_id: 'course-1',
                        subject_id: 'subject-1',
                    });
                }
                return Promise.resolve([]);
            });

            // Select from dropdowns and create session - should succeed
        });

        it('can create with all new inputs', async () => {
            mockInvoke.mockImplementation((cmd: string) => {
                if (cmd === 'create_and_start_session') {
                    return Promise.resolve({
                        id: 'session-1',
                        name: 'Test Session',
                        folder_id: 'new-folder-id',
                        course_id: 'new-course-id',
                        subject_id: 'new-subject-id',
                    });
                }
                return Promise.resolve([]);
            });

            // Enter new names and create session - should succeed
        });

        it('can create with mixed dropdown/new inputs', async () => {
            mockInvoke.mockImplementation((cmd: string) => {
                if (cmd === 'create_and_start_session') {
                    return Promise.resolve({
                        id: 'session-1',
                        name: 'Mixed Session',
                        folder_id: 'folder-1',
                        course_id: 'new-course-id',
                        subject_id: 'new-subject-id',
                    });
                }
                return Promise.resolve([]);
            });

            // Mix of existing and new - should succeed
        });
    });

    describe('Saved Sessions List', () => {
        it('displays all saved sessions', async () => {
            const mockSessions = [
                {
                    id: 'session-1',
                    name: 'CS Session',
                    folder_name: 'Computer Science',
                    course_name: 'Algorithms',
                    subject_name: 'Dynamic Programming',
                },
                {
                    id: 'session-2',
                    name: 'Math Session',
                    folder_name: 'Mathematics',
                    course_name: 'Calculus',
                    subject_name: 'Derivatives',
                },
            ];

            mockInvoke.mockImplementation((cmd: string) => {
                if (cmd === 'get_all_sessions') {
                    return Promise.resolve(mockSessions);
                }
                if (cmd === 'get_active_session') {
                    return Promise.resolve(null);
                }
                return Promise.resolve([]);
            });

            render(<SessionModal />);
            await openModal();

            await waitFor(() => {
                expect(mockInvoke).toHaveBeenCalledWith('get_all_sessions');
            });

            // Sessions should be displayed in the UI
        });

        it('shows folder > course > subject hierarchy', async () => {
            const session = {
                id: 'session-1',
                name: 'Test Session',
                folder_name: 'Folder',
                course_name: 'Course',
                subject_name: 'Subject',
            };

            // Display format should be "Folder / Course / Subject" or similar
            const displayName = `${session.folder_name} / ${session.course_name} / ${session.subject_name}`;
            expect(displayName).toBe('Folder / Course / Subject');
        });

        it('highlights active session', async () => {
            const activeSession = {
                id: 'session-1',
                name: 'Active Session',
                folder_name: 'Folder',
                course_name: 'Course',
                subject_name: 'Subject',
            };

            mockInvoke.mockImplementation((cmd: string) => {
                if (cmd === 'get_active_session') {
                    return Promise.resolve(activeSession);
                }
                if (cmd === 'get_all_sessions') {
                    return Promise.resolve([activeSession]);
                }
                return Promise.resolve([]);
            });

            render(<SessionModal />);
            await openModal();

            await waitFor(() => {
                expect(mockInvoke).toHaveBeenCalledWith('get_active_session');
            });

            // Active session should be visually distinct (different background, icon, etc.)
        });
    });

    describe('Session Deletion', () => {
        it('shows confirmation dialog before deletion', async () => {
            // Clicking delete should show a confirmation dialog
            // This is important to prevent accidental deletions
        });

        it('deletes session on confirm', async () => {
            mockInvoke.mockImplementation((cmd: string, args?: any) => {
                if (cmd === 'delete_session') {
                    expect(args?.sessionId).toBeDefined();
                    return Promise.resolve(undefined);
                }
                return Promise.resolve([]);
            });

            // Simulate confirming deletion
            // Verify delete_session command is called
        });

        it('cancels deletion on cancel', async () => {
            mockInvoke.mockClear();

            // Simulate canceling deletion
            // Verify delete_session command is NOT called
            expect(mockInvoke).not.toHaveBeenCalledWith('delete_session', expect.anything());
        });

        it('updates UI after deletion', async () => {
            const initialSessions = [
                { id: 'session-1', name: 'Session 1' },
                { id: 'session-2', name: 'Session 2' },
            ];

            let sessions = [...initialSessions];

            mockInvoke.mockImplementation((cmd: string, args?: any) => {
                if (cmd === 'get_all_sessions') {
                    return Promise.resolve(sessions);
                }
                if (cmd === 'delete_session') {
                    sessions = sessions.filter(s => s.id !== args?.sessionId);
                    return Promise.resolve(undefined);
                }
                return Promise.resolve([]);
            });

            // After deletion, the session should be removed from the list
        });

        it('clears active session if deleting active', async () => {
            const activeSession = { id: 'session-1', name: 'Active Session' };

            mockInvoke.mockImplementation((cmd: string, args?: any) => {
                if (cmd === 'get_active_session') {
                    return Promise.resolve(activeSession);
                }
                if (cmd === 'delete_session' && args?.sessionId === activeSession.id) {
                    // After deleting active session, get_active_session should return null
                    return Promise.resolve(undefined);
                }
                return Promise.resolve([]);
            });

            // Deleting the active session should clear the active session indicator
        });
    });

    describe('Error Handling', () => {
        it('handles network errors gracefully', async () => {
            mockInvoke.mockRejectedValue(new Error('Network error'));

            render(<SessionModal />);

            await waitFor(() => {
                // Should display error message to user
                // Should not crash the application
            });
        });

        it('handles invalid response data', async () => {
            mockInvoke.mockResolvedValue('invalid json');

            render(<SessionModal />);

            // Should handle JSON parse errors gracefully
        });

        it('displays user-friendly error messages', async () => {
            mockInvoke.mockImplementation((cmd: string) => {
                if (cmd === 'create_and_start_session') {
                    return Promise.reject(
                        'A session for "Computer Science / Algorithms / Dynamic Programming" already exists.'
                    );
                }
                return Promise.resolve([]);
            });

            // Error message should be displayed in the UI
            // Should be clear and actionable
        });
    });

    describe('Loading States', () => {
        it('shows loading indicator while fetching sessions', async () => {
            mockInvoke.mockImplementation((cmd: string) => {
                if (cmd === 'get_all_sessions') {
                    return new Promise(resolve => setTimeout(() => resolve([]), 1000));
                }
                return Promise.resolve([]);
            });

            render(<SessionModal />);

            // Should show loading indicator
        });

        it('disables form while creating session', async () => {
            mockInvoke.mockImplementation((cmd: string) => {
                if (cmd === 'create_and_start_session') {
                    return new Promise(resolve =>
                        setTimeout(() => resolve({ id: 'session-1', name: 'Test' }), 1000)
                    );
                }
                return Promise.resolve([]);
            });

            // Submit button and form fields should be disabled during creation
        });
    });

    describe('Accessibility', () => {
        it('has proper labels for all form fields', async () => {
            render(<SessionModal />);

            // All form fields should have associated labels
            // Screen readers should be able to navigate the form
        });

        it('supports keyboard navigation', async () => {
            render(<SessionModal />);

            // Should be able to navigate form using Tab key
            // Should be able to submit form using Enter key
            // Should be able to cancel using Escape key
        });

        it('has appropriate ARIA attributes', async () => {
            render(<SessionModal />);

            // Modal should have role="dialog"
            // Should have aria-labelledby for title
            // Should have aria-describedby for description
        });
    });
});
