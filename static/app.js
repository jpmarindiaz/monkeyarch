// State
let currentPath = '';

// Elements
const fileList = document.getElementById('file-list');
const breadcrumbs = document.getElementById('breadcrumbs');
const toast = document.getElementById('toast');

// Dialogs
const uploadDialog = document.getElementById('upload-dialog');
const mkdirDialog = document.getElementById('mkdir-dialog');
const renameDialog = document.getElementById('rename-dialog');
const deleteDialog = document.getElementById('delete-dialog');

// API helper
async function api(endpoint, options = {}) {
    const response = await fetch(endpoint, {
        ...options,
        headers: {
            'Content-Type': 'application/json',
            ...options.headers,
        },
    });

    const data = await response.json();

    if (!response.ok) {
        throw new Error(data.error || 'Request failed');
    }

    return data;
}

// Toast notifications
function showToast(message, isError = false) {
    toast.textContent = message;
    toast.className = isError ? 'error' : '';
    setTimeout(() => {
        toast.className = 'hidden';
    }, 3000);
}

// Format file size
function formatSize(bytes) {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
}

// Format date
function formatDate(dateStr) {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

// Get file icon
function getIcon(entry) {
    if (entry.type === 'directory') return '📁';
    const name = entry.name.toLowerCase();
    if (name.endsWith('.mp3')) return '🎵';
    if (name.match(/\.(jpg|jpeg|png|gif|webp)$/)) return '🖼️';
    return '📄';
}

// Render breadcrumbs
function renderBreadcrumbs(path) {
    const parts = path.split('/').filter(Boolean);
    let html = '<a href="#" data-path="">Home</a>';

    let currentBreadcrumb = '';
    parts.forEach((part, i) => {
        currentBreadcrumb += (currentBreadcrumb ? '/' : '') + part;
        html += ` <span>/</span> `;
        if (i === parts.length - 1) {
            html += `<span>${part}</span>`;
        } else {
            html += `<a href="#" data-path="${currentBreadcrumb}">${part}</a>`;
        }
    });

    breadcrumbs.innerHTML = html;

    // Add click handlers
    breadcrumbs.querySelectorAll('a').forEach(a => {
        a.addEventListener('click', (e) => {
            e.preventDefault();
            listDirectory(a.dataset.path);
        });
    });
}

// Render file list
function renderFileList(entries) {
    if (entries.length === 0) {
        fileList.innerHTML = '<div class="empty-state">Empty folder</div>';
        return;
    }

    fileList.innerHTML = entries.map(entry => `
        <div class="file-item" data-name="${entry.name}" data-type="${entry.type}">
            <span class="file-icon">${getIcon(entry)}</span>
            <div class="file-info">
                <div class="file-name">${entry.name}</div>
                <div class="file-meta">
                    ${entry.type === 'file' ? formatSize(entry.size || 0) : 'Folder'}
                    ${entry.modified ? ' • ' + formatDate(entry.modified) : ''}
                </div>
            </div>
            <div class="file-actions">
                <button type="button" class="rename-btn">Rename</button>
                <button type="button" class="delete-btn danger">Delete</button>
            </div>
        </div>
    `).join('');

    // Add click handlers
    fileList.querySelectorAll('.file-item').forEach(item => {
        const name = item.dataset.name;
        const type = item.dataset.type;

        // Click on item (navigate to directory)
        item.addEventListener('click', (e) => {
            if (e.target.closest('.file-actions')) return;
            if (type === 'directory') {
                const newPath = currentPath ? `${currentPath}/${name}` : name;
                listDirectory(newPath);
            }
        });

        // Rename button
        item.querySelector('.rename-btn').addEventListener('click', (e) => {
            e.stopPropagation();
            openRenameDialog(name);
        });

        // Delete button
        item.querySelector('.delete-btn').addEventListener('click', (e) => {
            e.stopPropagation();
            openDeleteDialog(name, type);
        });
    });
}

// List directory
async function listDirectory(path) {
    try {
        fileList.classList.add('loading');
        const data = await api(`/api/list?path=${encodeURIComponent(path)}`);
        currentPath = path;
        renderBreadcrumbs(path);
        renderFileList(data.entries);
    } catch (err) {
        showToast(err.message, true);
    } finally {
        fileList.classList.remove('loading');
    }
}

// Upload files
async function uploadFiles(files) {
    const formData = new FormData();
    for (const file of files) {
        formData.append('file', file);
    }

    const overwrite = document.getElementById('upload-overwrite').checked;
    const progress = document.getElementById('upload-progress');

    return new Promise((resolve, reject) => {
        const xhr = new XMLHttpRequest();

        xhr.upload.addEventListener('progress', (e) => {
            if (e.lengthComputable) {
                progress.value = (e.loaded / e.total) * 100;
            }
        });

        xhr.onload = () => {
            if (xhr.status === 200) {
                resolve();
            } else {
                try {
                    const err = JSON.parse(xhr.responseText);
                    reject(new Error(err.error || 'Upload failed'));
                } catch {
                    reject(new Error('Upload failed'));
                }
            }
        };

        xhr.onerror = () => reject(new Error('Network error'));

        xhr.open('POST', `/api/upload?path=${encodeURIComponent(currentPath)}&overwrite=${overwrite}`);
        xhr.send(formData);
    });
}

// Create directory
async function createDirectory(name) {
    const path = currentPath ? `${currentPath}/${name}` : name;
    await api('/api/mkdir', {
        method: 'POST',
        body: JSON.stringify({ path }),
    });
}

// Rename file/folder
async function renameItem(oldName, newName) {
    const from = currentPath ? `${currentPath}/${oldName}` : oldName;
    const to = currentPath ? `${currentPath}/${newName}` : newName;
    await api('/api/move', {
        method: 'POST',
        body: JSON.stringify({ from, to }),
    });
}

// Delete file/folder
async function deleteItem(name, recursive) {
    const path = currentPath ? `${currentPath}/${name}` : name;
    await api('/api/delete', {
        method: 'POST',
        body: JSON.stringify({ path, recursive }),
    });
}

// Dialog helpers
function openRenameDialog(name) {
    document.getElementById('rename-name').value = name;
    document.getElementById('rename-original').value = name;
    renameDialog.showModal();
}

function openDeleteDialog(name, type) {
    document.getElementById('delete-message').textContent =
        `Are you sure you want to delete "${name}"?`;
    document.getElementById('delete-path').value = name;
    const recursiveLabel = document.getElementById('delete-recursive-label');
    const recursiveCheck = document.getElementById('delete-recursive');
    recursiveLabel.style.display = type === 'directory' ? 'flex' : 'none';
    recursiveCheck.checked = false;
    deleteDialog.showModal();
}

// Event listeners
document.addEventListener('DOMContentLoaded', () => {
    // Initial load
    listDirectory('');

    // Upload button
    document.getElementById('btn-upload').addEventListener('click', () => {
        document.getElementById('file-input').value = '';
        document.getElementById('upload-progress').value = 0;
        document.getElementById('upload-overwrite').checked = false;
        uploadDialog.showModal();
    });

    // Upload form
    document.getElementById('upload-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const files = document.getElementById('file-input').files;
        if (files.length === 0) return;

        try {
            await uploadFiles(files);
            uploadDialog.close();
            showToast('Files uploaded successfully');
            listDirectory(currentPath);
        } catch (err) {
            showToast(err.message, true);
        }
    });

    // New folder button
    document.getElementById('btn-mkdir').addEventListener('click', () => {
        document.getElementById('mkdir-name').value = '';
        mkdirDialog.showModal();
    });

    // New folder form
    document.getElementById('mkdir-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const name = document.getElementById('mkdir-name').value.trim();
        if (!name) return;

        try {
            await createDirectory(name);
            mkdirDialog.close();
            showToast('Folder created');
            listDirectory(currentPath);
        } catch (err) {
            showToast(err.message, true);
        }
    });

    // Rename form
    document.getElementById('rename-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const original = document.getElementById('rename-original').value;
        const newName = document.getElementById('rename-name').value.trim();
        if (!newName || newName === original) {
            renameDialog.close();
            return;
        }

        try {
            await renameItem(original, newName);
            renameDialog.close();
            showToast('Renamed successfully');
            listDirectory(currentPath);
        } catch (err) {
            showToast(err.message, true);
        }
    });

    // Delete confirm
    document.getElementById('delete-confirm').addEventListener('click', async () => {
        const name = document.getElementById('delete-path').value;
        const recursive = document.getElementById('delete-recursive').checked;

        try {
            await deleteItem(name, recursive);
            deleteDialog.close();
            showToast('Deleted successfully');
            listDirectory(currentPath);
        } catch (err) {
            showToast(err.message, true);
        }
    });

    // Cancel buttons for all dialogs
    document.querySelectorAll('dialog .cancel').forEach(btn => {
        btn.addEventListener('click', () => {
            btn.closest('dialog').close();
        });
    });

    // Close dialogs on backdrop click
    document.querySelectorAll('dialog').forEach(dialog => {
        dialog.addEventListener('click', (e) => {
            if (e.target === dialog) {
                dialog.close();
            }
        });
    });
});
