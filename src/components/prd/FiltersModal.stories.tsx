import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import type { FilterSetters, FilterState } from "@/hooks/usePRDFilters";
import type { PriorityFilter, StatusFilter } from "@/types/prd";
import { FiltersModal } from "./FiltersModal";

const meta = {
  title: "Components/PRD/FiltersModal",
  component: FiltersModal,
  tags: ["autodocs"],
  args: {
    filters: { searchQuery: "", statusFilter: "all", priorityFilter: "all", tagFilter: "all" },
    setters: {
      setSearchQuery: () => {},
      setStatusFilter: () => {},
      setPriorityFilter: () => {},
      setTagFilter: () => {},
    },
    allTags: [],
    onClearFilters: () => {},
  },
} satisfies Meta<typeof FiltersModal>;

export default meta;
type Story = StoryObj<typeof meta>;

export const NoFilters: Story = {
  render: (args) => {
    const [searchQuery, setSearchQuery] = useState("");
    const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");
    const [priorityFilter, setPriorityFilter] = useState<PriorityFilter>("all");
    const [tagFilter, setTagFilter] = useState<string>("all");

    const filters: FilterState = {
      searchQuery,
      statusFilter,
      priorityFilter,
      tagFilter,
    };

    const setters: FilterSetters = {
      setSearchQuery,
      setStatusFilter,
      setPriorityFilter,
      setTagFilter,
    };

    return (
      <FiltersModal {...args} filters={filters} setters={setters} allTags={["ui", "api", "security", "testing"]} />
    );
  },
};

export const WithActiveFilters: Story = {
  render: (args) => {
    const [searchQuery, setSearchQuery] = useState("login");
    const [statusFilter, setStatusFilter] = useState<StatusFilter>("in_progress");
    const [priorityFilter, setPriorityFilter] = useState<PriorityFilter>("high");
    const [tagFilter, setTagFilter] = useState<string>("security");

    const filters: FilterState = {
      searchQuery,
      statusFilter,
      priorityFilter,
      tagFilter,
    };

    const setters: FilterSetters = {
      setSearchQuery,
      setStatusFilter,
      setPriorityFilter,
      setTagFilter,
    };

    return (
      <FiltersModal {...args} filters={filters} setters={setters} allTags={["ui", "api", "security", "testing"]} />
    );
  },
};

export const NoTags: Story = {
  render: (args) => {
    const [searchQuery, setSearchQuery] = useState("");
    const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");
    const [priorityFilter, setPriorityFilter] = useState<PriorityFilter>("all");
    const [tagFilter, setTagFilter] = useState<string>("all");

    const filters: FilterState = {
      searchQuery,
      statusFilter,
      priorityFilter,
      tagFilter,
    };

    const setters: FilterSetters = {
      setSearchQuery,
      setStatusFilter,
      setPriorityFilter,
      setTagFilter,
    };

    return <FiltersModal {...args} filters={filters} setters={setters} allTags={[]} />;
  },
};
