import { useParams, useNavigate } from 'react-router-dom';
import { Calendar, ExternalLink, Loader2 } from 'lucide-react';
import { motion } from 'framer-motion';
import { useNote } from '../hooks/useNotes';
import MarkdownRenderer from '../components/MarkdownRenderer';

const NotePage = () => {
  const { slug } = useParams<{ slug: string }>();
  const navigate = useNavigate();
  const { note, content, loading, error } = useNote(slug || '');

  if (!slug) {
    return <div>Invalid note ID</div>;
  }

  if (loading) {
    return (
      <motion.div
        className="flex justify-center items-center min-h-[400px]"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5 }}
      >
        <div className="flex flex-col items-center">
          <motion.div
            animate={{ rotate: 360 }}
            transition={{ duration: 2, repeat: Infinity, ease: 'linear' }}
          >
            <Loader2 className="h-12 w-12 text-primary-500 dark:text-primary-400" />
          </motion.div>
          <motion.p
            className="mt-4 text-gray-600 dark:text-gray-300"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.3 }}
          >
            Loading note...
          </motion.p>
        </div>
      </motion.div>
    );
  }

  if (error || !note) {
    return (
      <motion.div
        className="text-center py-10 max-w-md mx-auto"
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <div className="card border-red-400 dark:border-red-600 p-6">
          <motion.div
            className="text-red-500 mb-4"
            animate={{ scale: [1, 1.1, 1] }}
            transition={{ duration: 2, repeat: Infinity }}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-16 w-16 mx-auto"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
          </motion.div>
          <h2 className="text-2xl font-bold mb-3 text-gray-800 dark:text-gray-100">
            Failed to load note
          </h2>
          <p className="text-gray-600 dark:text-gray-300 mb-4">{error || 'Note not found'}</p>
          <motion.button
            className="mt-4 px-4 py-2 bg-primary-500 hover:bg-primary-600 text-white rounded-md shadow-md"
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
            onClick={() => navigate(-1)}
          >
            Go Back
          </motion.button>
        </div>
      </motion.div>
    );
  }

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.3 }}
      className="max-w-3xl mx-auto"
    >
      {/* Note content */}
      <motion.div
        className="mb-4"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.3 }}
      >
        <h1 className="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-2">{note.title}</h1>

        <div className="flex items-center text-sm text-gray-600 dark:text-gray-400 mb-6">
          <Calendar size={14} className="mr-1 text-gray-400" />
          {formatDate(note.last_modified)}
        </div>
      </motion.div>

      {/* Note content */}
      <motion.div
        className="prose dark:prose-invert max-w-none bg-white dark:bg-gray-800 p-4 rounded-lg shadow-sm"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.3 }}
      >
        <MarkdownRenderer content={content} />
      </motion.div>

      {/* Editor link */}
      <div className="mt-6 text-right">
        <button
          className="inline-flex items-center gap-1 text-gray-500 hover:text-primary-600 text-sm px-3 py-1"
          onClick={() => alert('To edit this note, use the TUI with command: nrs tui')}
        >
          <ExternalLink size={14} />
          <span>Edit</span>
        </button>
      </div>
    </motion.div>
  );
};

export default NotePage;
