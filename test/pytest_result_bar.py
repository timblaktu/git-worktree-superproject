"""pytest-result-bar - A pytest plugin that shows per-file progress bars with xdist support."""

import os
import sys
import threading
import time
from pathlib import Path

# Global storage for our reporter
_reporter = None
_lock = threading.Lock()


class DisplayManager:
    """Manages terminal display for progress bars."""
    
    def __init__(self):
        self.file_positions = {}  # {filename: line_position}
        self.terminal_width = self._get_terminal_width()
        self._cursor_hidden = False
        self.lock = threading.Lock()
        
    def _get_terminal_width(self):
        """Get terminal width with robust fallback."""
        try:
            import shutil
            width = shutil.get_terminal_size().columns
            return max(40, min(200, width))
        except (OSError, AttributeError, ImportError):
            try:
                width = int(os.environ.get('COLUMNS', 80))
                return max(40, min(200, width))
            except (ValueError, TypeError):
                return 80
                
    def hide_cursor(self):
        """Hide terminal cursor."""
        if not self._cursor_hidden:
            try:
                sys.stdout.write('\033[?25l')
                sys.stdout.flush()
                self._cursor_hidden = True
            except (OSError, IOError):
                pass
                
    def show_cursor(self):
        """Show terminal cursor."""
        if self._cursor_hidden:
            try:
                sys.stdout.write('\033[?25h')
                sys.stdout.flush()
                self._cursor_hidden = False
            except (OSError, IOError):
                pass
                
    def update_progress(self, filename, counts):
        """Update progress display for a file."""
        with self.lock:
            try:
                if filename not in self.file_positions:
                    # New file - assign next line position
                    self.file_positions[filename] = len(self.file_positions)
                    if len(self.file_positions) > 1:
                        sys.stdout.write('\n')  # Move to new line for new file
                        
                line_pos = self.file_positions[filename]
                progress_line = self._render_progress(filename, counts)
                
                # Move to correct line and update
                if line_pos > 0:
                    sys.stdout.write(f'\033[{line_pos}A')  # Move up
                    
                sys.stdout.write(f'\r\033[K{progress_line}')  # Clear line and write
                
                if line_pos > 0:
                    sys.stdout.write(f'\033[{line_pos}B')  # Move back down
                    
                sys.stdout.flush()
                
            except (OSError, IOError, UnicodeEncodeError):
                # Graceful degradation if terminal issues
                pass
                
    def _render_progress(self, filename, counts):
        """Render progress line for a file."""
        total = counts.get('total', 0)
        passed = counts.get('passed', 0)
        failed = counts.get('failed', 0)
        skipped = counts.get('skipped', 0)
        completed = passed + failed + skipped
        
        if total == 0:
            return f"{filename} [waiting for tests...]"
            
        # Calculate bar width
        filename_space = len(filename) + 1
        stats_space = 15  # Space for " 50/100" + safety margin
        bar_width = max(10, self.terminal_width - filename_space - stats_space)
        
        # Calculate progress
        progress = completed / total if total > 0 else 0
        filled = min(bar_width, max(0, round(progress * bar_width)))
        
        # Build colored segments
        if filled > 0 and completed > 0:
            # Calculate proportional widths
            passed_width = round((passed / completed) * filled) if completed > 0 else 0
            failed_width = round((failed / completed) * filled) if completed > 0 else 0
            skipped_width = filled - passed_width - failed_width
            
            # Ensure we don't exceed filled width
            if passed_width + failed_width + skipped_width > filled:
                if skipped_width > 0:
                    skipped_width = filled - passed_width - failed_width
                elif failed_width > 0:
                    failed_width = filled - passed_width
                    
            # Build bar with colors
            bar_parts = []
            if passed_width > 0:
                bar_parts.append('\033[32m' + '█' * passed_width)  # Green
            if failed_width > 0:
                bar_parts.append('\033[31m' + '█' * failed_width)  # Red
            if skipped_width > 0:
                bar_parts.append('\033[33m' + '█' * skipped_width)  # Yellow
                
            bar = ''.join(bar_parts) + '\033[0m' + '░' * (bar_width - filled)
        else:
            # Empty bar
            bar = '░' * bar_width
            
        return f"{filename} {bar} {completed}/{total}"
        
    def finalize_display(self):
        """Clean up display when done."""
        try:
            # Move cursor to end and add newline
            if self.file_positions:
                sys.stdout.write('\n')
                sys.stdout.flush()
        except (OSError, IOError):
            pass


class ResultBarReporter:
    """Main reporter class for progress bars."""
    
    def __init__(self, config):
        self.config = config
        self.is_controller = not hasattr(config, 'workerinput')
        self.test_counts = {}  # {filename: {total, passed, failed, skipped}}
        
        # Only controller handles display
        if self.is_controller:
            self.display_manager = DisplayManager()
        else:
            self.display_manager = None
            
    def start_collection(self):
        """Initialize display at start of collection."""
        if self.is_controller and self.display_manager:
            self.display_manager.hide_cursor()
            
    def finish_collection(self, session):
        """Count tests per file after collection."""
        if not self.is_controller:
            return

        # Count tests by file (only used in single mode, xdist uses node collection hook)
        for item in session.items:
            # Handle both fspath (older pytest) and path (newer pytest)
            if hasattr(item, 'fspath') and item.fspath:
                filename = os.path.basename(str(item.fspath))
            elif hasattr(item, 'path') and item.path:
                filename = os.path.basename(str(item.path))
            else:
                # Fall back to extracting from nodeid
                nodeid = getattr(item, 'nodeid', '')
                if '::' in nodeid:
                    filename = os.path.basename(nodeid.split('::')[0])
                else:
                    continue  # Can't determine filename
                    
            if filename not in self.test_counts:
                self.test_counts[filename] = {
                    'total': 0, 'passed': 0, 'failed': 0, 'skipped': 0
                }
            self.test_counts[filename]['total'] += 1
            
    def update_test_result(self, report):
        """Process a test result."""
        if not self.is_controller or report.when != 'call':
            return
            
        # Handle both fspath (older pytest) and path (newer pytest)
        if hasattr(report, 'fspath') and report.fspath:
            filename = os.path.basename(str(report.fspath))
        elif hasattr(report, 'path') and report.path:
            filename = os.path.basename(str(report.path))
        else:
            # Fall back to extracting from nodeid
            nodeid = getattr(report, 'nodeid', '')
            if '::' in nodeid:
                filename = os.path.basename(nodeid.split('::')[0])
            else:
                return  # Can't determine filename
        
        # Map test outcome
        if report.passed:
            outcome = 'passed'
        elif report.failed:
            outcome = 'failed'
        elif report.skipped:
            outcome = 'skipped'
        else:
            return  # Unknown outcome
            
        # Initialize file if not seen
        if filename not in self.test_counts:
            self.test_counts[filename] = {
                'total': 0, 'passed': 0, 'failed': 0, 'skipped': 0
            }
            
        # Update counts
        self.test_counts[filename][outcome] += 1
        
        # Keep total as maximum of current total and completed count
        completed = (self.test_counts[filename]['passed'] + 
                    self.test_counts[filename]['failed'] + 
                    self.test_counts[filename]['skipped'])
        
        # Dynamically update total - this will grow as we see more tests
        self.test_counts[filename]['total'] = max(self.test_counts[filename]['total'], completed)
            
        # Update display
        if self.display_manager:
            self.display_manager.update_progress(filename, self.test_counts[filename])
            
    def finalize(self):
        """Clean up at end of session."""
        if self.is_controller and self.display_manager:
            self.display_manager.finalize_display()
            self.display_manager.show_cursor()


def pytest_addoption(parser):
    """Add plugin options."""
    group = parser.getgroup('terminal reporting')
    group.addoption(
        '--result-bar',
        action='store_true',
        default=False,
        help='Show per-file progress bars instead of dots'
    )


def pytest_configure(config):
    """Configure the plugin."""
    global _reporter
    
    # Enforce loadfile distribution mode for optimal per-file progress
    if not hasattr(config, 'workerinput'):  # Only in controller
        if (hasattr(config.option, 'numprocesses') and 
            config.option.numprocesses and 
            hasattr(config.option, 'dist')):
            config.option.dist = 'loadfile'
    
    # Enable plugin if --result-bar is set
    if config.getoption('--result-bar'):
        with _lock:
            _reporter = ResultBarReporter(config)
            config._result_bar_reporter = _reporter
            config._result_bar_enabled = True
            
        # Suppress default output to avoid interference
        if not hasattr(config, 'workerinput'):  # Only in controller
            config.option.verbose = -1
            config.option.no_header = True
            
            # Disable terminal reporter features
            terminal_reporter = config.pluginmanager.get_plugin('terminalreporter')
            if terminal_reporter:
                terminal_reporter.showfspath = False
                terminal_reporter.showlongtestinfo = False
                terminal_reporter.reportchars = ''


def pytest_collection_finish(session):
    """Called after collection is finished."""
    global _reporter
    if _reporter:
        _reporter.start_collection()
        _reporter.finish_collection(session)


def pytest_runtest_logreport(report):
    """Process test results in real-time."""
    global _reporter
    if _reporter:
        _reporter.update_test_result(report)


def pytest_sessionfinish(session):
    """Handle session finish."""
    global _reporter
    if _reporter:
        _reporter.finalize()


def pytest_report_teststatus(report, config):
    """Suppress default test status output."""
    if hasattr(config, '_result_bar_enabled'):
        # Return empty strings to suppress dots/letters
        return report.outcome, '', ''


def pytest_xdist_node_collection_finished(node, ids):
    """Called when a worker node finishes collecting tests."""
    global _reporter
    if _reporter and _reporter.is_controller:
        # Only use the first worker's collection to avoid double counting
        # All workers collect the same tests, so we only need one
        if not hasattr(_reporter, '_collection_initialized'):
            _reporter._collection_initialized = True
            
            # Count tests by file from the collected IDs
            for test_id in ids:
                if '::' in test_id:
                    filename = os.path.basename(test_id.split('::')[0])
                    if filename not in _reporter.test_counts:
                        _reporter.test_counts[filename] = {
                            'total': 0, 'passed': 0, 'failed': 0, 'skipped': 0
                        }
                    _reporter.test_counts[filename]['total'] += 1


def pytest_terminal_summary(terminalreporter):
    """Print final test summary."""
    global _reporter
    
    if not _reporter or not _reporter.is_controller:
        return
        
    # Calculate totals
    total_passed = sum(counts.get('passed', 0) for counts in _reporter.test_counts.values())
    total_failed = sum(counts.get('failed', 0) for counts in _reporter.test_counts.values())
    total_skipped = sum(counts.get('skipped', 0) for counts in _reporter.test_counts.values())
    
    # Print summary
    parts = []
    if total_passed:
        parts.append(f"{total_passed} passed")
    if total_failed:
        parts.append(f"{total_failed} failed")
    if total_skipped:
        parts.append(f"{total_skipped} skipped")
        
    if parts:
        try:
            print(f"\nResults: {', '.join(parts)}")
        except (OSError, IOError):
            pass