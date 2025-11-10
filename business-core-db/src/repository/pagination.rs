/// Pagination request parameters for offset-based pagination
/// 
/// # Example
/// ```
/// use business_core_db::repository::pagination::PageRequest;
/// 
/// let page_request = PageRequest::new(20, 0); // First page with 20 items
/// let next_page = PageRequest::new(20, 20); // Second page
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageRequest {
    /// Maximum number of items to return
    pub limit: usize,
    /// Number of items to skip
    pub offset: usize,
}

impl PageRequest {
    /// Create a new page request
    /// 
    /// # Arguments
    /// * `limit` - Maximum number of items to return
    /// * `offset` - Number of items to skip
    pub fn new(limit: usize, offset: usize) -> Self {
        Self { limit, offset }
    }

    /// Create a page request for a specific page number (1-based)
    /// 
    /// # Arguments
    /// * `page_size` - Number of items per page
    /// * `page_number` - Page number (1-based, will be converted to 0-based offset)
    /// 
    /// # Example
    /// ```
    /// use business_core_db::repository::pagination::PageRequest;
    /// 
    /// let page_1 = PageRequest::for_page(20, 1); // offset: 0
    /// let page_2 = PageRequest::for_page(20, 2); // offset: 20
    /// ```
    pub fn for_page(page_size: usize, page_number: usize) -> Self {
        let page_number = page_number.max(1); // Ensure page_number is at least 1
        Self {
            limit: page_size,
            offset: (page_number - 1) * page_size,
        }
    }

    /// Get the page number (1-based) for this request
    pub fn page_number(&self) -> usize {
        if self.limit == 0 {
            1
        } else {
            (self.offset / self.limit) + 1
        }
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            limit: 20,
            offset: 0,
        }
    }
}

/// Paginated response containing items and metadata
/// 
/// # Example
/// ```
/// use business_core_db::repository::pagination::Page;
/// 
/// let page = Page {
///     items: vec![1, 2, 3],
///     total: 100,
///     limit: 20,
///     offset: 0,
/// };
/// 
/// assert_eq!(page.has_more(), true);
/// assert_eq!(page.page_number(), 1);
/// assert_eq!(page.total_pages(), 5);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    /// The items in this page
    pub items: Vec<T>,
    /// Total number of items across all pages
    pub total: usize,
    /// Maximum number of items per page
    pub limit: usize,
    /// Number of items skipped before this page
    pub offset: usize,
}

impl<T> Page<T> {
    /// Create a new page
    /// 
    /// # Arguments
    /// * `items` - The items in this page
    /// * `total` - Total number of items across all pages
    /// * `limit` - Maximum number of items per page
    /// * `offset` - Number of items skipped before this page
    pub fn new(items: Vec<T>, total: usize, limit: usize, offset: usize) -> Self {
        Self {
            items,
            total,
            limit,
            offset,
        }
    }

    /// Check if there are more pages after this one
    pub fn has_more(&self) -> bool {
        self.offset + self.items.len() < self.total
    }

    /// Get the current page number (1-based)
    pub fn page_number(&self) -> usize {
        if self.limit == 0 {
            1
        } else {
            (self.offset / self.limit) + 1
        }
    }

    /// Get the total number of pages
    pub fn total_pages(&self) -> usize {
        if self.limit == 0 {
            1
        } else {
            self.total.div_ceil(self.limit)
        }
    }

    /// Check if this is the first page
    pub fn is_first_page(&self) -> bool {
        self.offset == 0
    }

    /// Check if this is the last page
    pub fn is_last_page(&self) -> bool {
        !self.has_more()
    }
}