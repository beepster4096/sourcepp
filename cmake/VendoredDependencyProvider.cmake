include_guard(GLOBAL)

set(SOURCEPP_VENDORED_PATH "" CACHE PATH "Path containing vendored dependencies")
option(SOURCEPP_VENDORED_PREPARE "Instead of providing vendored dependencies, save all dependencies to vendor later" OFF)

if("${SOURCEPP_VENDORED_PATH}" STREQUAL "")
    message(FATAL_ERROR "missing SOURCEPP_VENDORED_PATH")
endif()

function(sourcepp_vendored_provide_dependency method dep_name)
    cmake_parse_arguments(PARSE_ARGV 2 arg "EXCLUDE_FROM_ALL;SYSTEM" "BINARY_DIR" "")
    string(TOLOWER ${dep_name} dep_name_lower)
    
    FetchContent_GetProperties(${dep_name} POPULATED populated)
    if(populated)
        return()
    endif()

    cmake_path(APPEND dep_source_dir ${SOURCEPP_VENDORED_PATH} ${dep_name_lower})

    if(SOURCEPP_VENDORED_PREPARE)
        FetchContent_MakeAvailable(${dep_name})

        message(STATUS "Vendoring ${dep_name}")
        file(GLOB source_dir_contents "${${dep_name_lower}_SOURCE_DIR}/*")
        file(REMOVE ${dep_source_dir})
        file(COPY ${source_dir_contents} DESTINATION ${dep_source_dir})

        return()
    endif()

    if(NOT EXISTS ${dep_source_dir})
        message(FATAL_ERROR "Dependency ${dep_name} was not vendored!")
    endif()

    if(EXISTS "${dep_source_dir}/CMakeLists.txt")
        set(add_subdirectory_args ${dep_source_dir} ${arg_BINARY_DIR})
        if(arg_EXCLUDE_FROM_ALL)
            list(APPEND add_subdirectory_args EXCLUDE_FROM_ALL)
        endif()
        if(arg_SYSTEM)
            list(APPEND add_subdirectory_args EXCLUDE_FROM_ALL)
        endif()

        add_subdirectory(${add_subdirectory_args})
    endif()

    FetchContent_SetPopulated(
        ${dep_name}
        SOURCE_DIR ${dep_source_dir}
        BINARY_DIR ${arg_BINARY_DIR}
    )
endfunction()

cmake_language(
    SET_DEPENDENCY_PROVIDER sourcepp_vendored_provide_dependency
    SUPPORTED_METHODS FETCHCONTENT_MAKEAVAILABLE_SERIAL
)