import { Link } from "react-router-dom";
import { Calendar, Tag } from "lucide-react";
import { motion } from "framer-motion";
import { Note } from "../types";

interface NoteCardProps {
  note: Note;
}

const NoteCard = ({ note }: NoteCardProps) => {
  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleDateString();
  };

  return (
    <motion.div
      whileHover={{ y: -5 }}
      transition={{ type: "spring", stiffness: 300 }}
    >
      <Link
        to={`/notes/${note.slug}`}
        className="card h-full flex flex-col bg-white dark:bg-gray-800 overflow-hidden shadow-md hover:shadow-lg transition-all duration-200"
      >
        <div className="flex-1">
          <h2 className="text-lg font-semibold text-gray-800 dark:text-gray-100 mb-2 border-b border-gray-100 dark:border-gray-700 pb-2">
            {note.title}
          </h2>

          <p className="text-gray-600 dark:text-gray-300 text-sm line-clamp-4 mb-3">
            {note.preview || "No preview available"}
          </p>

          {note.tags && note.tags.length > 0 && (
            <div className="flex flex-wrap gap-1.5 mb-3">
              {note.tags.map((tag, index) => (
                <motion.span
                  key={tag}
                  className="tag flex items-center gap-1"
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ delay: index * 0.05, duration: 0.2 }}
                  whileHover={{ scale: 1.1 }}
                >
                  <Tag className="w-3 h-3" />
                  {tag}
                </motion.span>
              ))}
            </div>
          )}
        </div>

        <div className="text-xs text-gray-500 dark:text-gray-400 flex items-center gap-1 mt-auto pt-2 border-t border-gray-100 dark:border-gray-700">
          <Calendar className="w-3 h-3" />
          <span>Last updated: {formatDate(note.last_modified)}</span>
        </div>
      </Link>
    </motion.div>
  );
};

export default NoteCard;
