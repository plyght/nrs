import { useParams, Link, useNavigate } from "react-router-dom";
import {
  ChevronLeft,
  Tag,
  Calendar,
  ExternalLink,
  Loader2,
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useNote } from "../hooks/useNotes";
import MarkdownRenderer from "../components/MarkdownRenderer";

const NotePage = () => {
  const { slug } = useParams<{ slug: string }>();
  const navigate = useNavigate();
  const { note, content, loading, error } = useNote(slug || "");

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
            transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
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
          <p className="text-gray-600 dark:text-gray-300 mb-4">
            {error || "Note not found"}
          </p>
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
    return date.toLocaleDateString() + " " + date.toLocaleTimeString();
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.5 }}
    >
      {/* Back button */}
      <motion.div
        className="mb-3"
        initial={{ x: -20, opacity: 0 }}
        animate={{ x: 0, opacity: 1 }}
        transition={{ duration: 0.4 }}
      >
        <motion.div
          whileHover={{ x: -5 }}
          transition={{ type: "spring", stiffness: 400 }}
        >
          <Link
            to="/"
            className="inline-flex items-center text-primary-500 hover:text-primary-700 font-medium"
          >
            <ChevronLeft size={16} />
            <span>Back to notes</span>
          </Link>
        </motion.div>
      </motion.div>

      {/* Note header - combined with header information */}
      <motion.div
        className="card mb-4 border-l-4 border-primary-500 dark:border-primary-400"
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.5, delay: 0.1 }}
      >
        <div className="flex flex-col space-y-2">
          <motion.h1
            className="text-2xl font-bold text-gray-800 dark:text-gray-100"
            initial={{ x: -10, opacity: 0 }}
            animate={{ x: 0, opacity: 1 }}
            transition={{ duration: 0.5, delay: 0.2 }}
          >
            {note.title}
          </motion.h1>

          <motion.div
            className="flex flex-wrap gap-3 text-sm text-gray-600 dark:text-gray-400"
            initial={{ y: 10, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            transition={{ duration: 0.5, delay: 0.3 }}
          >
            <motion.div
              className="flex items-center gap-1"
              whileHover={{ scale: 1.05 }}
            >
              <Calendar size={14} className="text-primary-400" />
              Last modified: {formatDate(note.last_modified)}
            </motion.div>

            {note.tags && note.tags.length > 0 && (
              <motion.div
                className="flex items-center gap-2"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 0.4 }}
              >
                <Tag size={14} className="text-primary-400" />
                <div className="flex flex-wrap gap-1.5">
                  <AnimatePresence>
                    {note.tags.map((tag, index) => (
                      <motion.div
                        key={tag}
                        initial={{ opacity: 0, y: 10 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ delay: 0.5 + index * 0.1 }}
                      >
                        <motion.div
                          whileHover={{ scale: 1.1, y: -2 }}
                          whileTap={{ scale: 0.95 }}
                        >
                          <Link
                            to={`/?search=${encodeURIComponent(tag)}`}
                            className="tag hover:bg-primary-200 dark:hover:bg-primary-800 transition-all duration-300"
                          >
                            {tag}
                          </Link>
                        </motion.div>
                      </motion.div>
                    ))}
                  </AnimatePresence>
                </div>
              </motion.div>
            )}
          </motion.div>
        </div>
      </motion.div>

      {/* Note content */}
      <motion.div
        className="card overflow-hidden"
        initial={{ y: 30, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.6, delay: 0.4 }}
      >
        <MarkdownRenderer content={content} />
      </motion.div>

      {/* Editor link */}
      <motion.div
        className="mt-8 text-center"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5, delay: 0.6 }}
      >
        <p className="text-sm text-gray-500 dark:text-gray-400 mb-3">
          This is a read-only view
        </p>
        <motion.button
          className="inline-flex items-center gap-2 text-primary-500 hover:text-primary-600 dark:hover:text-primary-300 text-sm font-medium px-4 py-2 rounded-full bg-primary-50 dark:bg-primary-900/30 hover:bg-primary-100 dark:hover:bg-primary-800/50 transition-colors duration-300"
          onClick={() =>
            alert("To edit this note, use the TUI with command: nrs tui")
          }
          whileHover={{ scale: 1.05, y: -2 }}
          whileTap={{ scale: 0.95 }}
          transition={{ type: "spring", stiffness: 400 }}
        >
          <ExternalLink size={16} />
          Open in editor
        </motion.button>
      </motion.div>
    </motion.div>
  );
};

export default NotePage;
